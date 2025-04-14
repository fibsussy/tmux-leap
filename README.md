This script will give you a command `tmux-leap` and an UI to switch betwween sessions better with fzf

# Installation

Arch Linux Install
```sh
curl -sSL https://raw.githubusercontent.com/fibsussy/tmux-leap/refs/heads/main/arch_install.sh | sh
```


### Recommended binds for tmux

no tmux prefix ctrl+f
```sh
bind-key -n C-f popup -E -d '#{pane_current_path}' 'tmux-leap'
```
tmux prefix ctrl+f
```sh
bind-key C-f popup -E -d '#{pane_current_path}' 'tmux-leap'
```
