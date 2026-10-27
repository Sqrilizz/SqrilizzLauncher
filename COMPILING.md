# Prerequisites

Make sure you have Rust and Cargo installed (https://rustup.rs).

The GUI might need some additional dependencies on Linux (such as `fontconfig`, `wayland` or `x11` development libraries).

# Compiling the CLI

To compile the CLI:

```sh
cargo build -p sqrilizz_cli --release
```

The built binary will be located at `target/release/sqrilizz`.

To run in development mode:

```sh
cargo run -p sqrilizz_cli -- <args>
```

To install `sqrilizz` locally:

```sh
cargo install --path crates/cli --locked
```

# Compiling the GUI

To build the native desktop app:

```sh
cargo build -p sqrilizz_launcher --release
```

The executable will be located at `target/release/sqrilizz_launcher`.

To run the GUI in development mode:

```sh
cargo run -p sqrilizz_launcher
```

# Running Tests

To run the complete test suite across all workspace crates:

```sh
cargo test --workspace
```
