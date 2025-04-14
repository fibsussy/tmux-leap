# tmux-leap 🚀

**Supercharge your tmux workflow with lightning-fast session management**

tmux-leap is a powerful CLI tool that lets you instantly navigate between tmux sessions using fuzzy search. Stop wasting time typing long session names or scrolling through lists - just leap to where you need to be!

## ✨ Features

- **Blazing Fast Navigation**: Instantly jump between tmux sessions with fuzzy search
- **Smart Session Management**: Automatically creates sessions for directories that don't have one
- **Project Tracking**: Maintains a list of your favorite directories for quick access
- **Recursive Directory Support**: Scan subdirectories at configurable depths
- **Intelligent Caching**: Remembers your most frequently used sessions
- **Seamless Integration**: Works both inside and outside of tmux

## 🔧 Installation

### Arch Linux

```bash
curl -sSL https://raw.githubusercontent.com/fibsussy/tmux-leap/refs/heads/main/arch_install.sh | sh
```

### Manual Installation (from source)

```bash
# Clone the repository
git clone https://github.com/fibsussy/tmux-leap.git
cd tmux-leap

# Build and install
cargo build --release
sudo cp target/release/tmux-leap /usr/local/bin/
```

## 🚀 Getting Started

### Basic Usage

Simply run `tmux-leap` to see a fuzzy finder with all your available sessions and projects:

```bash
tmux-leap
```

### Adding Projects

Add your current directory to the projects list:

```bash
tmux-leap add
```

Or specify a directory:

```bash
tmux-leap add ~/projects/awesome-project
```

### Managing Projects

List all your projects:

```bash
tmux-leap list
```

Delete a project (interactive):

```bash
tmux-leap delete
```

Edit your projects file directly:

```bash
tmux-leap edit
```

### Advanced Features

Set a depth to recursively include subdirectories:

```bash
tmux-leap set-depth
```

## ⌨️ Recommended tmux Keybindings

Add one of these to your `~/.tmux.conf` for quick access:

### Option 1: No tmux prefix (Ctrl+F)

```bash
bind-key -n C-f popup -E -d '#{pane_current_path}' 'tmux-leap'
```

### Option 2: With tmux prefix (Prefix+Ctrl+F)

```bash
bind-key C-f popup -E -d '#{pane_current_path}' 'tmux-leap'
```

## 🔍 How It Works

tmux-leap maintains a list of your projects in `~/.projects` and intelligently combines them with existing tmux sessions. When you select a project:

1. If a tmux session already exists for that directory, it switches to it
2. If no session exists, it creates a new one and attaches to it
3. Your most frequently used sessions are cached for faster access

## 🛠️ Configuration

Your projects are stored in `~/.projects` with a simple format:

```
~/projects/awesome-project
~/work/client-project --depth 2
~/personal/blog
```

The `--depth` option allows you to include subdirectories up to the specified depth.

## 📚 Command Reference

| Command | Description |
|---------|-------------|
| `tmux-leap` | Main command - shows fuzzy finder |
| `tmux-leap add [dir]` | Add current or specified directory |
| `tmux-leap delete` | Remove a project (interactive) |
| `tmux-leap list` | List all projects |
| `tmux-leap status` | Show raw projects file content |
| `tmux-leap set-depth` | Set recursive depth for a project |
| `tmux-leap edit` | Edit projects file in your default editor |
| `tmux-leap completion <shell>` | Generate shell completions |

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
