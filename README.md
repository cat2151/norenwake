# norenwake

A TUI (Terminal User Interface) for Windows that helps you effectively "norenwake" (spin off) your own public GitHub repository into a new, independent one. Written in Rust.

## Previous Challenges and This App's Solution

| Previous Challenges                                                                      | This App's Solution                                                                                                                                                                                            |
| :--------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| High cognitive load for a series of tasks, leading to fatigue.                           | Visualizes the workflow with a TUI, reducing cognitive load.                                                                                                                                                   |
| Time-consuming to find the target repository.                                            | Quickly find the target repo with `/` filtering (space-separated AND search).                                                                                                                                  |
| Concern about accidental pushes because the remote still points to the original repo after cloning. | Immediately reconfigures remotes after cloning, deleting `upstream`. Unifies `origin`'s fetch to HTTPS and push to SSH.                                                                                        |
| Configuration and README often become inconsistent when changing to a new repo name.     | Changing the repo name with `n` updates the working directory name, remote settings, and `README.ja.md`'s leading header all at once.                                                                             |
| Time-consuming to confirm changes.                                                       | You can check README preview and diff (delta) within the screen, and logs are also retained.                                                                                                                   |
| Hard to notice misconfigurations before pushing.                                         | Open the verification screen with `Shift + P` to check `origin` / push URL / `upstream`. If in a dangerous state, a hard guard rejects the push.                                                                |

The shortest procedure for practical use is: `Enter` to clone → `n` to name → `c` to commit → `Shift + P` to verify → `y` to push.
Proceeding step-by-step within the TUI reduces the risk of errors or omissions and lowers cognitive load.

## Safety Features

- Only "your own owner repositories" that are "public / non-fork / non-archived" are displayed as clone sources.
- Immediately reconfigures remotes to a safer state after cloning.
- Deletes `upstream` and sets `origin`'s fetch URL to HTTPS and push URL to SSH.
- Performs a hard guard before pushing.
- Rejects push if `origin` points to the norenwake source repository.
- Rejects push if `upstream` remains.

## Requirements

- Rust (`cargo`)
- Python
- `git`
- `gh` (GitHub CLI)
- `delta` (diff tool)

Python is only required for `norenwake update`'s self-update process, as it delegates to `cat-self-update-lib`.

`gh` is used for API authentication. Tokens are retrieved in the following priority order:

1. `GH_TOKEN`
2. `GITHUB_TOKEN`
3. `gh auth token`

## Installation

```bash
cargo install --force --git https://github.com/cat2151/norenwake
```

## Startup

```bash
norenwake
```

## Update

```bash
norenwake update
```

## Check for Updates

```bash
norenwake check
```

## Key Operations

- `h` / `l` / `←` / `→`: Move focus pane (repos / dir tree / log)
- `j` / `k` / `↑` / `↓`: Move within current pane
- `PageUp` / `PageDown`: Page move within current pane
- `Enter` (on repos pane): Clone selected repo
- `n`: Edit new repo name
- `c`: Commit
- `Shift + P`: Pre-push verification and push confirm
- `Shift + L`: Copy full log to clipboard
- `/`: Open repos filtering overlay (space-separated AND search)
- `?`: Help overlay
- `q`: Quit

## Workflow

1. Select target in `repos` and `Enter` to clone
2. Edit and confirm `new repo name` with `n`
3. Commit with `c`
4. Check verification results with `Shift + P`
5. Press `y` to push

## README Update Rules

`update_readme_ja` maintains only one of the following "norenwake" headers at the beginning of `README.ja.md`:

```md
# <new repo name>

元repoからcloneして暖簾分けしました。暖簾分け断面までの履歴を持っています。
```

If existing headers of the same type are consecutive, they are collapsed, leaving no duplicates.

## README Preview

- Retrieval priority is `README.ja.md`, falling back to `README.md` if it fails.

## Data Storage Location (Windows)

- `%LOCALAPPDATA%\norenwake\`

## Assumption
- This is an app for personal use, not intended for others. If you want similar functionality, we recommend cloning or creating your own.

## What This App Aims For
- PoC. To demonstrate (and has demonstrated) that it's possible to create a helpful personal app with Codex.

## What This App Does Not Aim For (Out of Scope)
- Support. Responding to requests or suggestions.