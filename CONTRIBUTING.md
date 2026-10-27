# Contributing to Sqrilizz

Contributions, issues, and feature requests are welcome!

## Project Structure

- `/` and `/src`: The root of the project and the `sqrilizz` crate. Contains library code for instance templates, setup, and configuration.
- `/crates`: Core modules and applications:
  - `/crates/gui`: The native desktop graphical application (`sqrilizz_launcher`).
  - `/crates/cli`: The command-line interface (`sqrilizz`).
  - `/crates/core`: Core launcher logic, Java runtime management, and game launching.
  - `/crates/auth`: Account management and authentication (Microsoft, offline).
  - `/crates/config`: Configuration deserialization and management.
  - `/crates/instance`: Instance management, storage, file locks, and logs.
  - `/crates/mods`: Modloader integrators (Fabric, Forge, NeoForge, Quilt, Paper).
  - `/crates/pkg`: Standard formats, repository management, and declarative package resolution.
  - `/crates/pkg_gen`: Package scrapers and metadata generators for Modrinth and Smithed.
  - `/crates/parse`: Package script and condition parsing.
  - `/crates/shared`: Shared data structures, networking helpers, and common utilities.
  - `/crates/options`: Generation of Minecraft options files in a backwards-compatible manner.
  - `/crates/tools`: Command-line utilities for schema generation and maintenance.
- `/tools`: Build, packaging, and release automation scripts.
