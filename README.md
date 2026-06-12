# F1 Stalker

> [!NOTE] 
> GitLab is the official home of the app! https://gitlab.com/aaanh/f1-stalker
> GitHub is the mirror: http://github.com/aaanh/f1-stalker

You can't get enough of Formula One? You live and breathe the drama, the bottling, and the comedy? Then you're probably just like me. That's why I built this app to track the results and upcoming races.

And if you're a sadistically masochistic, you would have built it in Rust, like me.

> [!NOTE] 
> Attribution: F1 Stalker relies on the data made available by [OpenF1 API](https://github.com/br-g/openf1), so maybe consider buying them a coffee. And if you like this app, buy me 1 and I'll match 0.5x to OpenF1!

# Installation

Please check out the Releases page of this project to find the right distribution for your OS.

I currently support:

- macOS >= Sonoma ARM64 (first-class) -- on which I primarily built this thing.
- Linux AMD64 (AppImage), requires libfuse2 iirc.
- Windows AMD64, cross compiled, tested on my Windows 10 and 11 rigs.

## Build from source

If you want to build from source, you'll need to install {cargo/rust, git}, then you can follow these steps.

- Rust >= 1.96

These commands are to be run inside a terminal emulator (Terminal, GNOME Terminal, PowerShell, etc.)

1. Clone the repository

  ```sh
  git clone git@gitlab.com:aaanh/f1-stalker
  ```

2. Change directory and run

  ```sh
  cd f1-stalker
  cargo run
  ```

  Optionally, run with hot reload on source change

  ```sh
  cargo watch
  ```

# Contribution

> [!WARNING] 
> Github is a read-only mirror. Pull requests (PR) and issues submitted on Github are not monitored, much like how the `torvalds/linux` repository works.

Contributions come in 2 flavors: pull requests and issue submissions.

## Pull Requests

Please fork, branch, make changes, and open a PR into `master` branch of this project.

Commit message convention: 
- Format: `{feat,fix,chore,docs}({some_component,some_feature}): concise action description`
- Example: `feat(sys): optimize app startup process by async loading data` or `fix(net): solve a race condition when using tokio`

## Issues

If you encounter an issue or a bug, please raise an issue with the following information:

- OS-level
  - e.g. Windows XP Professional 32-bit, Gentoo AMD64 Kernel v6.9.0-rc with sway/wayland, or the output of `uname -ar` + `cat /etc/os-release`. `fastfetch/screenfetch/neofetch` output is acceptable
- Step-by-step reproduction, if it's not consistent or not immediately reproducible, do your best effort to write down what you remember.
- (Optional) your theory on what happened, what caused it, how it could be solved.

# Community Guidelines

Please be respectful. Jokes and shenanigans are tolerated... to a certain point. Keep it classy.

I do my **best-effort** to resolve issues needing code changes.
