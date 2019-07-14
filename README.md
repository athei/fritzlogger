# fritzlogger

This is a a service daemon you can run our your computer or Raspberry Pi in order to continuously
log data from the Fritz!Box attached home automation device sensors. This is useful when you want
to do your own data analysis on the raw data. The Fritz!Box itself does not store the historical
data we need for that.

# Installation
Rust programmers can simply:
```
cargo install fritzlogger
```

# Usage
```
# Generate a default config
fritzlogger defconfig > conf.toml
# Run it after tweaking the config
fritzlogger run -c conf.toml
```

# Building
fritzlogger is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org) in order to compile it.

After installing Rust you can build it:
```
git clone https://github.com/athei/fritzlogger.git
cd fritzlogger
cargo build --release
./target/release/fritzlogger --version
fritzlogger 0.1.0
```