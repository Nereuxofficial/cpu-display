# CPU-Display
> This houses RP2040 Firmware written in Rust for displaying CPU usage externally on a 64x32 Matrix. Basically for each core it randomly fills the core area based on the core utilization in red.

## Building & Flashing
To build the project you need a working [Rust installation](https://rustup.rs/).
To build the project run:
```
cd benchy
cargo b -r # short for build --release
```

The flashing is set up for a debug probe using [probe-rs](https://probe.rs/), however it can also be done without it
### Using a debug probe
Run:
```
# Assuming you are still in the benchy folder
cargo r -r # short for run --release
```
### Without a debug probe
First install elf2uf2-rs via this command:
```
cargo install elf2uf2-rs
```
Edit the file `benchy/.cargo/config.toml` to replace the runner line with this:
```
runner = "elf2uf2-rs -d"
```
And now plug in your RP2040 board while holding down the `RESET` button.
Then run the command:
```
cargo r -r
```
And the firmware should be flashed and then your RP2040 board should reboot.



## Credits
Inspired by the little connection machine from Adafruit: https://learn.adafruit.com/little-connection-machine?embeds=allow
Wiring: https://blog.benson.zone/2023/12/bevy-game-of-life/
