# norenwake

A TUI for Windows that creates a new repository by 'norenwake' (branching off) your own public GitHub repositories. Written in Rust.

## Previous Challenges and This App's Solutions

| Previous Challenges | This App's Solution |
| --- | --- |
| High cognitive load for a series of tasks, leading to fatigue | Visualized with TUI, reducing cognitive load. |
| Time-consuming to find the target repository | Quickly find the target repository with `/` filtering (space-separated AND search). |
| Concern about accidental pushes because the remote still points to the original repository after cloning | Immediately after cloning, reconfigure the remote and delete `upstream`. `origin`'s fetch URL is unified to HTTPS, and push URL to SSH. |
| Settings and README tend to become inconsistent when changing to a new repository name | When changing the repository name with `n`, the working directory name, remote settings, and `README.ja.md`'s leading header are updated together. |
| Time-consuming to confirm changes | README preview and diff (delta) can be confirmed on screen, and logs are also retained. |
| Difficult to notice configuration errors before pushing | Open the validation screen with `Shift + P` to check `origin` / push URL / `upstream`. If in a dangerous state, push is rejected with a hard guard. |

The shortest procedure for practical operation is: `Enter` to clone → `n` to name → `c` to commit → `Shift + P` to validate → `y` to push.
By proceeding step-by-step within the TUI, the risk of errors or omissions and cognitive load can be reduced.

## Safety Features

- Only "your own owner repositories" that are "public / non-fork / non-archived" are displayed as clone sources.
- Immediately after cloning, the remote is reconfigured to the safe side.
- `upstream` is deleted, and `origin`'s fetch URL is set to HTTPS, push URL to SSH.
- A hard guard is implemented before pushing.
- If `origin` points to the 'norenwake' source, push is rejected.
- If `upstream` remains, push is rejected.

## Requirements

- Rust (`cargo`)
- `git`
- `gh` (GitHub CLI)
- `delta` (diff tool)

`gh` is used for API authentication. Tokens are retrieved in the following priority order:

1. `GH_TOKEN`
2. `GITHUB_TOKEN`
3. `gh auth token`

## Installation

```bash
cargo install --force --git https://github.com/cat2151/norenwake
```

## Usage

```bash
norenwake
```

## Update

```bash
norenwake update
```

## Keybindings

- `h` / `l` / `←` / `→`: Move focus pane (repos / dir tree / log)
- `j` / `k` / `↑` / `↓`: Move within the current pane
- `PageUp` / `PageDown`: Page move within the current pane
- `Enter` (on repos pane): Clone selected repository
- `n`: Edit new repo name
- `c`: Commit
- `Shift + P`: Pre-push validation and push confirmation
- `Shift + L`: Copy full log to clipboard
- `/`: Open repos filter overlay (space-separated AND search)
- `?`: Help overlay
- `q`: Quit

## Workflow

1. Select target in `repos` and `Enter` to clone
2. Edit and confirm `new repo name` with `n`
3. Commit with `c`
4. Confirm validation results with `Shift + P`
5. Press `y` to push

## README Update Rules

`update_readme_ja` maintains only one of the following 'norenwake' headers at the beginning of `README.ja.md`.

```md
# <new repo name>

Cloned from the original repo as a 'norenwake'. It holds the history up to the 'norenwake' point.
```

If existing headers of the same type are consecutive, they are folded, and no duplicates are left.

## README Preview

- `README.ja.md` is prioritized for retrieval; if it fails, it falls back to `README.md`.

## Data Storage Location (Windows)

- `%LOCALAPPDATA%\norenwake\`

## Assumption
- This app is for personal use and is not intended for others. If you want similar functionality, we recommend cloning or creating your own.

## What This App Aims For
- PoC. To demonstrate that a helpful personal app can be created with Codex (demonstrated).

## What This App Does Not Aim For (Out of Scope)
- Support. Responding to requests or suggestions.