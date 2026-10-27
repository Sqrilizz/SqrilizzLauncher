# CI and Automation

Here are some tips and information for running Sqrilizz in CI pipelines, Docker containers, or any other automated system.

## Installing

To install the CLI, which is the most useful for automation, download the binary directly from GitHub releases or compile with `cargo install --path crates/cli`.

## Skipping Onboarding

The first time the CLI is run after being installed, it may show an initial onboarding prompt. To skip this in non-interactive CI environments, pass the `--skip-onboarding` flag:

```sh
sqrilizz --skip-onboarding instance list
```

## Auto-Yes

For commands like `launch` and `instance update`, there can sometimes be confirmation prompts for package installs or upgrades.

To automatically confirm these prompts, add `--yes`:

```sh
sqrilizz --yes instance update foo
```

## Running as Root

When running Sqrilizz as root on Linux, the data directory will be `/root/.local/share/sqrilizz`, not `/usr/share/sqrilizz`. The config directory will be `/root/.config/sqrilizz`.
