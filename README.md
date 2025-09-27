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

## roadmap
- make a volume control interface and see how far a song played ☑️
- add sink management 🏗️  -> moved to other project
- add playback to sink control logic and management interface 🏗️  -> moved to other project
- fix the ISSUE on created file: -> You have to manually delete the config with a new version if config things changed
- add config for refresh rate

## Changelog

### V 2.2.0
- fixed dbus spam (uses 2024 rust to compile and latest glibc for debian trixie)

### V 2.1.0
- bumped dbus cargo dependency fixing errors

### V 2.0.0
- initial release
- buggy in terms of streaming
