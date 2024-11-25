# marstui-rustio
this is a wonderful tool to manage your audio from a terminal no more pavucontrol


## building fails if not using these packages:
sudo apt install libdbus-1-dev pkg-config


```
git clone https://github.com/schooldanlp6/marstui-rustio
cd marstui-rustio
cargo build --release
cd target/release
chmod +x marstui-audio
sudo cp marstui-audio /bin/marstui-audio
```
## Small documentation
The configuration is in ~/.config/marstui/audio.toml

This is not supposed to be an interfacing library but can be treated as such and is in this repo: [Private]
