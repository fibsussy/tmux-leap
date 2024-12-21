This script will give you a command `jumper` and an UI to switch betwween sessions better with fzf

![image](https://github.com/user-attachments/assets/ec013d6b-b027-423a-bc1b-d7a4e721cb15)
![image](https://github.com/user-attachments/assets/722859d1-0d2f-42d3-819c-8ca1e44c1c7c)



# Installation

Arch Linux Install
```sh
curl -sSL https://raw.githubusercontent.com/Fibalious/jumper/refs/heads/main/arch_install.sh | sh
```


### Recommended binds for tmux

no tmux prefix ctrl+f
```sh
bind-key -n C-f popup -E -d '#{pane_current_path}' 'jumper'
```
tmux prefix ctrl+f
```sh
bind-key C-f popup -E -d '#{pane_current_path}' 'jumper'
```
