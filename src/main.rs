pub mod tmux;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use dirs::home_dir;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, thread};
extern crate whoami;

#[derive(Debug, Parser)]
#[command(name = "Jumper", about = "fzf through a list of projects")]
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
    /// Clear the cache file
    #[command(name = "clear-cache", aliases = &["cc"])]
    ClearCache,
    /// Generate shell completion scripts
    #[command(name = "completion", aliases = &["comp", "c"])]
    Completion {
        /// The shell to generate the script for (e.g., bash, zsh, fish, powershell, elvish)
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone)]
struct Project {
    path: String,
}

impl Project {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    fn to_fzf_display(&self) -> String {
        let user = whoami::username();
        self.path
            .replace(&format!("/home/{user}"), "~")
            .replace("/run/media/fib/ExternalSSD/code", "code")
            .replace('.', "")
    }

    fn exists(&self) -> bool {
        let path = PathBuf::from(&self.path);
        path.exists() && path.is_dir()
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
        Some(Commands::Add { dir }) => add_project(dir.as_deref()),
        Some(Commands::Delete) => delete_project(),
        Some(Commands::List) => list_projects(),
        Some(Commands::Status) => status_projects(),
        Some(Commands::SetDepth) => set_depth(),
        Some(Commands::ClearCache) => clear_cache(),
        Some(Commands::Completion { shell }) => generate_completion(shell),
        None => main_execution(),
    }
}

fn generate_completion(shell: Shell) {
    let mut cmd = Opt::command();
    let bin_name = env!("CARGO_PKG_NAME");
    generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}

fn get_home_path(file: &str) -> PathBuf {
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

fn add_project(dir: Option<&str>) {
    let projects_file = get_home_path(".projects");
    touch_file(&projects_file);
    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
    let dir = dir.unwrap_or(&current_dir).to_string();
    let mut lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    if !lines.contains(&dir) {
        lines.push(dir.clone());
    }
    write_lines(&projects_file, &lines).unwrap();
    println!("Added \"{dir}\" to .projects");
}

fn delete_project() {
    let projects_file = get_home_path(".projects");
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    let mut selected = Command::new("fzf")
        .arg("--reverse")
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
        .iter_mut()
        .map(|i| Project::new(i))
        .collect()
}

fn get_projects() -> Vec<Project> {
    let projects_file = get_home_path(".projects");
    let mut projects = Vec::new();
    let mut unique_projects = HashSet::new();

    // Load projects from the .projects file
    if let Ok(lines) = read_lines(&projects_file) {
        let re = Regex::new(r"(.*) --depth (\d+)").unwrap();
        for line in lines {
            if let Some(captures) = re.captures(&line) {
                let dir = captures.get(1).unwrap().as_str();
                let depth = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
                projects.push(Project::new(dir));
                let sub_dirs = Command::new("find")
                    .arg("-L")
                    .arg(dir)
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

    // Add tmux sessions to the projects list
    projects.extend(get_tmux_sessions());

    // Filter out duplicates
    projects
        .into_iter()
        .filter(|project| unique_projects.insert(project.path.clone()))
        .collect()
}

fn reorder_projects_by_history(history: &[String], projects: &[Project]) -> Vec<Project> {
    let mut reordered_projects = Vec::new();
    let mut seen = HashSet::new();
    let projects_map: HashMap<String, &Project> =
        projects.iter().map(|p| (p.to_fzf_display(), p)).collect();

    // Add projects from history first (most recent first)
    for hist in history.iter().rev() {
        if let Some(project) = projects_map.get(hist) {
            if seen.insert(project.path.clone()) {
                reordered_projects.push((*project).clone());
            }
        }
    }

    // Add remaining projects that are not in the history
    for project in projects {
        if seen.insert(project.path.clone()) {
            reordered_projects.push(project.clone());
        }
    }

    reordered_projects
}

fn move_to_tmux_session(dir: &Project) {
    let tmux_session_name_og = dir.to_fzf_display();
    let tmux_session_name = tmux_session_name_og.replace('~', "\\~");

    if !tmux::session_exists(&tmux_session_name_og)
        && !tmux::create_session(&tmux_session_name_og, &dir.path)
    {
        eprintln!("Failed to create new tmux session");
        return;
    }

    if tmux::is_inside_tmux() {
        if !tmux::switch_client(&tmux_session_name) {
            eprintln!("Failed to switch tmux client");
        }
    } else {
        if !tmux::attach_session(&tmux_session_name) {
            eprintln!("Failed to attach to tmux session");
        }
    }
}

fn main_execution() {
    let projects_history_file = get_home_path(".projects_history");
    touch_file(&projects_history_file);
    let history_lines = read_lines(&projects_history_file).unwrap_or_else(|_| vec![]);

    let temp_file = PathBuf::from("/tmp/jumper_temp_file");
    File::create(&temp_file).expect("failed to create temp file");

    let fzf_process = start_fzf(&temp_file);

    let temp_file_clone = temp_file.clone();
    let history_lines_clone = history_lines.clone();
    thread::spawn(move || {
        let reordered_projects = load_and_filter_projects(&history_lines_clone);
        let fzf_through = prepare_fzf_content(&reordered_projects, &history_lines_clone);
        write_lines(&temp_file_clone, &fzf_through).unwrap();
    });

    let selected_str = wait_for_fzf_selection(fzf_process);
    if selected_str.is_empty() {
        println!("No selection made");
        return;
    }

    update_history(&projects_history_file, &history_lines, &selected_str);

    let reordered_projects = load_and_filter_projects(&history_lines);
    move_to_selected_tmux_session(&reordered_projects, &selected_str);

    if temp_file.exists() {
        remove_file(&temp_file).expect("Failed to remove temporary file");
    }
}

fn start_fzf(temp_file: &PathBuf) -> std::process::Child {
    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "tail -f {} | fzf --layout=reverse --no-border --cycle --extended",
            temp_file.display()
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute fzf")
}

fn load_and_filter_projects(history_lines: &[String]) -> Vec<Project> {
    // Always fetch new projects to ensure we have the latest list
    let new_projects = get_projects();

    // Update the cache with the new projects
    let cache_file = PathBuf::from("/tmp/.projects_cache");
    let project_paths: Vec<String> = new_projects.iter().map(|p| p.path.clone()).collect();
    write_lines(&cache_file, &project_paths).unwrap();

    // Filter out invalid projects
    let valid_projects = new_projects.filter_exists();

    // Reorder projects based on history
    reorder_projects_by_history(history_lines, &valid_projects)
}

fn prepare_fzf_content(reordered_projects: &[Project], history_lines: &[String]) -> Vec<String> {
    let current_session = tmux::get_current_session();
    let mut fzf_through: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    // Add history items first (most recent first)
    for item in history_lines.iter().rev() {
        // Skip the current session
        if let Some(current_session) = &current_session {
            if item == current_session {
                continue;
            }
        }

        // Check if the project exists before adding it to fzf_through
        if let Some(project) = reordered_projects
            .iter()
            .find(|p| p.to_fzf_display() == *item)
        {
            if project.exists() && seen.insert(item.clone()) {
                fzf_through.push(item.clone());
            }
        }
    }

    // Add remaining projects that aren't in the history
    for project in reordered_projects {
        let display = project.to_fzf_display();

        // Skip the current session
        if let Some(current_session) = &current_session {
            if &display == current_session {
                continue;
            }
        }

        if seen.insert(display.clone()) {
            fzf_through.push(display);
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

fn update_history(projects_history_file: &PathBuf, history_lines: &[String], selected_str: &str) {
    let mut new_history = vec![selected_str.to_string()];
    new_history.extend(
        history_lines
            .iter()
            .filter(|&item| item != selected_str)
            .cloned(),
    );
    new_history.truncate(2000);
    write_lines(projects_history_file, &new_history).unwrap();
}

fn move_to_selected_tmux_session(reordered_projects: &[Project], selected_str: &str) {
    if let Some(idx) = reordered_projects
        .iter()
        .position(|p| p.to_fzf_display() == selected_str)
    {
        let dir = reordered_projects.get(idx).unwrap();
        move_to_tmux_session(dir);
    } else {
        println!("L");
    }
}

fn list_projects() {
    let projects = get_projects();
    for project in projects {
        println!("{}", project.path);
    }
}

fn status_projects() {
    let projects_file = get_home_path(".projects");
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    for line in lines {
        println!("{line}");
    }
}

fn set_depth() {
    let projects_file = get_home_path(".projects");
    let lines = read_lines(&projects_file).unwrap_or_else(|_| vec![]);
    let mut selected = Command::new("fzf")
        .arg("--reverse")
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

fn clear_cache() {
    let cache_file = PathBuf::from("/tmp/.projects_cache");
    if cache_file.exists() {
        remove_file(&cache_file).expect("Failed to delete cache file");
        println!("Cache cleared");
    } else {
        println!("No cache file found");
    }
}
