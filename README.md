# **Pixel editor**
A pixel art editor written in Rust, inspired by LibreSprite and Aseprite.
> **Status:** early prototype; the sprite model is still being worked out, all of this *probably* will have a change or two.

## **Goals**
- A pixel editor with layers, frames and palette-based (indexed) color.
- A custom, documented and hardened file format.
- Longer term; the same team plans to build a 2D game engine on top of this.

## **Building**
You need a recent stable Rust toolchain (the project uses the 2024 edition, so Rust 1.85 or newer).
```
cargo build
cargo test
cargo run --example heart
cargo run --example nyan
cargo run --example pacman
```
Do note that these tests are meant to render / play these gifs via ANSI; this best works on the Windows Terminal or Kitty, most likely won't work in plain CMD.

## **Contributing**
I would put something here, but SOMEONE has only ran `touch CONTRIBUTING.md` and nothing else. So there is not much to say here, be patient :3

## **License**
This software is provided under the MIT license which you can read [here](LICENSE.md)!