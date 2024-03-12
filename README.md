# swtchr

A pretty Gnome-style window switcher for the [Sway](https://swaywm.org/) window
manager.

## Building from source

This crate requires native dependencies to build.

The easiest way to install these dependencies locally is to clone the repo and
use the provided nix shell. [Install nix](https://nixos.org/download) and then
run:

```shell
git clone https://github.com/lostatc/swtchr
cd ./swtchr
nix-shell
```

You can also use [direnv](https://direnv.net) to load the nix shell
automatically:

```shell
cd ./swtchr
direnv allow
```

If you would rather install the necessary dependencies yourself:

- [GTK 4](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_linux.html)
- [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell?tab=readme-ov-file#distro-packages)
- [librsvg](https://gitlab.gnome.org/GNOME/librsvg)

To build the `swtchrd` daemon and `swtchr` client, [install
Rust](https://www.rust-lang.org/tools/install) and run:

```shell
cargo build --workspace --release
```

You can find the generated binaries here:

- `./target/release/swtchr`
- `./target/release/swtchrd`
