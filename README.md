This script will give you a command `jumper` and an UI to switch betwween sessions better with fzf


# Installation

Arch Linux Install
```sh
curl -s https://raw.githubusercontent.com/Fibalious/jumper/refs/heads/main/arch_install.sh | bash
```

Recommended bind for tmux
```sh
bind-key -n C-f popup -E -d '#{pane_current_path}' 'jumper'
```



