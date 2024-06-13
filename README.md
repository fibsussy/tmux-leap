This is intended to use with some sort of overlay system built in your teminal. It will attach (not switch) to the tmux session wherever you run it.

If you use kitty, here is the snippet:
```
map ctrl+f launch --cwd=current --type=overlay ~/Scripts/jumper
```
here is the snippet to put the binary in that folder:
```
mkdir -p ~/Scripts/
cargo build --release
ln -s $(pwd)/target/release/jumper ~/Scripts/jumper
```

If you use a different terminal... idk what to tell you man lol
