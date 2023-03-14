[![Deploy latest](https://github.com/hoangph271/hbp/actions/workflows/deploy-latest.yml/badge.svg)](https://github.com/hoangph271/hbp/actions/workflows/deploy-latest.yml)

> To be abandoned `Rocket.rs` website

It's actually live [here](https://sneu.date/)

---

## Issues:
- Pretty much only work on Linux & macOS
- Not working on a blockchain (that was the original plan)

## Build:

- Debian/Ubuntu based distros:

```bash
# Install required packages:
sudo apt install build-essential pkg-config libssl-dev -y

# Release build:
cargo build --release
```

## Setup:

- Initialize the `.env` file and the `Rocket.toml` file
- Run `cargo run .`

## Preview:

![image](static/images/hbp.png)
