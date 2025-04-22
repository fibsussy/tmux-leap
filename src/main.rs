pub mod tmux;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use dirs::home_dir;
use regex::Regex;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, thread};
use tempfile::NamedTempFile;
extern crate whoami;

const PROJECTS_FILE: &str = ".projects";
const CACHE_FILE: &str = ".projects_cache";
const FZF_LAYOUT: &str = "--layout=reverse --no-border --cycle --extended";
const MAX_CACHE_ENTRIES: usize = 100;

#[derive(Debug, Parser)]
#[command(name = "tmux-leap", about = "fzf through a list of directories")]
struct Opt {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a project to the .projects file
    #[command(name = "add", aliases = &["a"])]
    Add {
        /// The project directory to add. If not provided, the current directory will be added.
        dir: Option<String>,
        
        /// Set a recursive depth for subdirectories
        #[arg(long)]
        depth: Option<u32>,
    },
    /// Delete a project from the .projects file
    #[command(name = "delete", aliases = &["del", "d"])]
    Delete,
    /// List all projects in the .projects file
    #[command(name = "list", aliases = &["ls", "l"])]
    List,
    /// Display the contents of the .projects file
    #[command(name = "status", aliases = &["stat", "s"])]
    Status,
    /// Set or remove depth for a project
    #[command(name = "set-depth", aliases = &["depth", "sd"])]
    SetDepth,
    /// Edit the .projects file in your default editor
    #[command(name = "edit", aliases = &["e"])]
    Edit,
    /// Generate shell completion scripts
    #[command(name = "completion", aliases = &["comp", "c"])]
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone)]
struct Project {
    shortened_path: String,
    expanded_path: String,
    tmux_display_path: String,
}

impl Project {
    fn new(path: &str) -> Self {
        let expanded_path = shellexpand::tilde(&path)
            .to_string()
            .trim_end_matches('/')
            .to_string();
        let shortened_path = Self::shorten_path(&expanded_path)
            .trim_end_matches('/')
            .to_string();
        let tmux_display_path = Self::format_for_tmux(&shortened_path)
            .trim_end_matches('/')
            .to_string();

        Self {
            shortened_path,
            expanded_path,
            tmux_display_path,
        }
    }

    fn shorten_path(path_str: &str) -> String {
        let path = PathBuf::from(path_str);
        let home = home_dir().expect("Unable to find home directory");

        if let Ok(relative) = path.strip_prefix(&home) {
            format!("~/{}", relative.display())
        } else {
            path.display().to_string()
        }
    }

    fn format_for_tmux(path: &str) -> String {
        path.replace('.', "_")
    }

    fn to_fzf_display(&self) -> &str {
        &self.shortened_path
    }

    fn exists(&self) -> bool {
        let path = PathBuf::from(&self.expanded_path);
        path.exists() && path.is_dir()
    }

    fn attach(&self) {
        let tmux_session_name = &self.tmux_display_path;
        
        let session_exists = tmux::session_exists(tmux_session_name);
        
        if !session_exists {
            if !tmux::create_session(tmux_session_name, &self.expanded_path) {
                eprintln!("Failed to create new tmux session");
                return;
            }
        }
        
        if tmux::is_inside_tmux() {
            let escaped_name = if tmux_session_name == "~" {
                "\\~".to_string()
            } else {
                tmux_session_name.to_string()
            };
            
            if !tmux::switch_client(&escaped_name) {
                eprintln!("Failed to switch tmux client");
            }
        } else if !tmux::attach_session(tmux_session_name) {
            eprintln!("Failed to attach to tmux session");
        }
    }
}

trait FilterExists {
    fn filter_exists(&self) -> Vec<Project>;
}

impl FilterExists for Vec<Project> {
    fn filter_exists(&self) -> Vec<Project> {
        self.iter()
            .filter(|project| project.exists())
            .cloned()
            .collect()
    }
}

fn main() {
    let opt = Opt::parse();
    match opt.command {
        Some(Commands::Add { dir, depth }) => add_project(dir.as_deref(), depth),
        Some(Commands::Delete) => delete_project(),
        Some(Commands::List) => list_projects(),
        Some(Commands::Status) => status_projects(),
        Some(Commands::SetDepth) => set_depth(),
        Some(Commands::Edit) => edit_projects_file(),
        Some(Commands::Completion { shell }) => generate_completion(shell),
        None => execution(),
    }
}

fn generate_completion(shell: Shell) {
    let mut cmd = Opt::command();
    let bin_name = env!("CARGO_PKG_NAME");
    generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}

fn get_home_path(file: &str) -> PathBuf {
    let mut file = file;
    if file.starts_with('~') {
        file = file.strip_prefix("~").unwrap();
    }
    home_dir()
        .expect("Unable to find home directory")
        .join(file)
}

fn touch_file(path: &PathBuf) {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .unwrap();
}

fn read_lines<P>(filename: P) -> std::io::Result<Vec<String>>
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?;
    let buf = BufReader::new(file);
    buf.lines().collect()
}

fn write_lines<P>(filename: P, lines: &[String]) -> std::io::Result<()>
where
    P: AsRef<std::path::Path>,
{
    let mut file = File::create(filename)?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    Ok(())
}

fn add_project(dir: Option<&str>, depth: Option<u32>) {
    let projects_file = get_home_path(PROJECTS_FILE);
    touch_file(&projects_file);
    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
    let dir = dir.unwrap_or(&current_dir).to_string();
    let project = Project::new(&dir);
    let mut lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    
    // Remove any existing entry for this path
    lines.retain(|line| {
        let re = Regex::new(r"(.*) --depth \d+").unwrap();
        if let Some(captures) = re.captures(line) {
            let path = captures.get(1).unwrap().as_str();
            path != project.shortened_path
        } else {
            line != &project.shortened_path
        }
    });
    
    // Add the path with depth if specified
    if let Some(depth_value) = depth {
        lines.push(format!("{} --depth {}", project.shortened_path, depth_value));
        println!("Added \"{}\" to .projects with depth {}", project.shortened_path, depth_value);
    } else {
        lines.push(project.shortened_path.clone());
        println!("Added \"{}\" to .projects", project.shortened_path);
    }
    
    write_lines(&projects_file, &lines).unwrap();
}

fn delete_project() {
    let projects_file = get_home_path(PROJECTS_FILE);
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    let mut selected = Command::new("fzf")
        .args(FZF_LAYOUT.split_whitespace())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute fzf");
    {
        let fzf_stdin = selected.stdin.as_mut().expect("Failed to open fzf stdin");
        fzf_stdin
            .write_all(lines.join("\n").as_bytes())
            .expect("Failed to write to fzf stdin");
    }
    let output = selected
        .wait_with_output()
        .expect("Failed to read fzf output");
    if !output.stdout.is_empty() {
        let selected_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let new_lines: Vec<String> = lines
            .into_iter()
            .filter(|line| line != &selected_str)
            .collect();
        write_lines(&projects_file, &new_lines).unwrap();
        println!("Deleted \"{selected_str}\" from .projects");
    }
}

fn get_tmux_sessions() -> Vec<Project> {
    tmux::get_sessions()
        .iter()
        .map(|i| Project::new(i))
        .collect()
}

fn get_projects() -> Vec<Project> {
    let projects_file = get_home_path(PROJECTS_FILE);
    let mut projects = Vec::new();
    let mut unique_projects = HashSet::new();
    if let Ok(lines) = read_lines(&projects_file) {
        let re = Regex::new(r"(.*) --depth (\d+)").unwrap();
        for line in lines {
            if let Some(captures) = re.captures(&line) {
                let dir = captures.get(1).unwrap().as_str();
                let depth = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                projects.push(Project::new(dir));
                let project = Project::new(dir);
                let sub_dirs = Command::new("find")
                    .arg("-L")
                    .arg(&project.expanded_path)
                    .arg("-maxdepth")
                    .arg(depth.to_string())
                    .arg("-type")
                    .arg("d")
                    .output()
                    .expect("Failed to execute find");
                let sub_dirs = String::from_utf8_lossy(&sub_dirs.stdout);
                for sub_dir in sub_dirs.lines() {
                    projects.push(Project::new(sub_dir));
                }
            } else {
                projects.push(Project::new(&line));
            }
        }
    }
    projects.extend(get_tmux_sessions());
    projects
        .into_iter()
        .filter(|project| unique_projects.insert(project.expanded_path.clone()))
        .collect()
}

fn prepare_fzf_content_from_cache(cache_file: &PathBuf, temp_file: &PathBuf) -> Vec<String> {
    let mut output_file = OpenOptions::new()
        .append(true)
        .open(temp_file)
        .expect("Failed to open temp file for appending");
    let current_session = tmux::get_current_session();
    read_lines(cache_file)
        .unwrap_or_else(|_| vec![])
        .into_iter()
        .map(|line| Project::new(&line))
        .filter(|project| {
            current_session
                .as_ref()
                .is_none_or(|session| project.tmux_display_path != *session)
        })
        .filter(Project::exists)
        .scan(HashSet::new(), |seen, project| {
            if seen.insert(project.expanded_path.clone()) {
                writeln!(output_file, "{}", project.to_fzf_display())
                    .expect("Failed to write to temp file");
                Some(project.shortened_path.clone())
            } else {
                None
            }
        })
        .collect()
}

fn execution() {
    let cache_file = get_home_path(CACHE_FILE);
    touch_file(&cache_file);
    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    let temp_path = temp_file.path().to_path_buf();
    let cache_lines = prepare_fzf_content_from_cache(&cache_file, &temp_path);
    let fzf_process = start_fzf(&temp_path);
    let mut seen_items: HashSet<String> = cache_lines.into_iter().collect();
    let temp_path_clone = temp_path;
    thread::spawn(move || {
        let projects = load_and_filter_projects();
        let additional_fzf_through = prepare_fzf_content(&projects);
        let mut file = OpenOptions::new()
            .append(true)
            .open(&temp_path_clone)
            .expect("Failed to open temp file for appending");
        for item in additional_fzf_through {
            if seen_items.insert(item.clone()) {
                writeln!(file, "{item}").expect("Failed to write to temp file");
            }
        }
    });
    let selected_str = wait_for_fzf_selection(fzf_process);
    {
        let cleanup_result = cleanup(&cache_file, &selected_str);
        if let Err(e) = cleanup_result {
            eprintln!("Cleanup failed: {e}");
        }
    }
    if selected_str.is_empty() {
        println!("No selection made");
        return;
    }
    let selected_project = Project::new(&selected_str);
    load_and_filter_projects()
        .iter()
        .find(|p| p.expanded_path == selected_project.expanded_path)
        .map(|p| p.attach())
        .expect("Selected project not found");
}

fn cleanup(cache_file: &PathBuf, selected_str: &str) -> std::io::Result<()> {
    if !selected_str.is_empty() {
        let mut cache_lines = read_lines(cache_file).unwrap_or_else(|_| vec![]);
        cache_lines.retain(|line| line != selected_str);
        cache_lines.insert(0, selected_str.to_string());
        if cache_lines.len() > MAX_CACHE_ENTRIES {
            cache_lines.truncate(MAX_CACHE_ENTRIES);
        }
        write_lines(cache_file, &cache_lines)?;
    }
    Ok(())
}

fn start_fzf(temp_file: &PathBuf) -> std::process::Child {
    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tail -f -n +0 {} | fzf {}",
            temp_file.display(),
            FZF_LAYOUT
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute fzf")
}

fn load_and_filter_projects() -> Vec<Project> {
    get_projects().filter_exists()
}

fn prepare_fzf_content(projects: &[Project]) -> Vec<String> {
    let mut fzf_through: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for project in projects {
        let display = project.to_fzf_display();
        if seen.insert(project.expanded_path.clone()) {
            fzf_through.push(display.to_string());
        }
    }
    fzf_through
}

fn wait_for_fzf_selection(fzf_process: std::process::Child) -> String {
    let output = fzf_process
        .wait_with_output()
        .expect("Failed to read fzf output");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn list_projects() {
    let projects = get_projects();
    for project in projects {
        println!("{}", project.shortened_path);
    }
}

fn status_projects() {
    let projects_file = get_home_path(PROJECTS_FILE);
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    for line in lines {
        println!("{line}");
    }
}

fn set_depth() {
    let projects_file = get_home_path(PROJECTS_FILE);
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    let mut selected = Command::new("fzf")
        .args(FZF_LAYOUT.split_whitespace())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute fzf");
    {
        let fzf_stdin = selected.stdin.as_mut().expect("Failed to open fzf stdin");
        fzf_stdin
            .write_all(lines.join("\n").as_bytes())
            .expect("Failed to write to fzf stdin");
    }
    let output = selected
        .wait_with_output()
        .expect("Failed to read fzf output");
    if !output.stdout.is_empty() {
        let selected_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("Set depth for {selected_str}: (Press Enter to remove depth, Ctrl+C to cancel)");
        let mut depth_input = String::new();
        std::io::stdin()
            .read_line(&mut depth_input)
            .expect("Failed to read depth input");
        let depth_input = depth_input.trim();
        let re = Regex::new(r"(.*) --depth \d+").unwrap();
        let mut new_lines: Vec<String> = lines
            .into_iter()
            .filter(|line| !re.is_match(line) || !line.starts_with(&selected_str))
            .collect();
        if depth_input.is_empty() {
            new_lines.push(selected_str.clone());
        } else {
            new_lines.push(format!("{selected_str} --depth {depth_input}"));
        }
        new_lines.sort();
        write_lines(&projects_file, &new_lines).unwrap();
        println!("Set depth for \"{selected_str}\" to {depth_input}");
    }
}

fn edit_projects_file() {
    let projects_file = get_home_path(PROJECTS_FILE);
    touch_file(&projects_file);
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor)
        .arg(projects_file)
        .status()
        .expect("Failed to launch editor");
    if !status.success() {
        eprintln!("Editor exited with non-zero status: {}", status);
    }
}
