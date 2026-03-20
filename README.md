# norenwake

A TUI for Windows to create new repositories by branching off your public GitHub repositories. Written in Rust.

## Previous Challenges and This App's Solutions

| Previous Challenges                                                   | This App's Solution                                                                                                                                                                                                                                     |
| :-------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| High cognitive load for a series of tasks, leading to fatigue.        | Visualized with a TUI to reduce cognitive load.                                                                                                                                                                                                         |
| Time-consuming to find the target repository.                         | Quickly find the target repo using `/` for filtering (space-separated AND search).                                                                                                                                                                      |
| Risk of accidental pushes because the remote still points to the original repo after cloning. | Reconfigure remotes immediately after cloning, deleting `upstream`. Unify `origin`'s fetch to HTTPS and push to SSH.                                                                                                                   |
| Configuration and README can easily become inconsistent when changing to a new repo name. | Changing the repo name with `n` updates the working directory name, remote settings, and the `README.ja.md` top header all at once.                                                                                                |
| Time-consuming to confirm changes.                                    | View README preview and diff (delta) on screen, and logs are also retained.                                                                                                                                                                             |
| Difficult to detect misconfigurations before pushing.                 | Open the verification screen with `Shift + P` to check `origin` / push URL / `upstream`. If in a dangerous state, a hard guard rejects the push.                                                                                                          |

The shortest operational procedure is: `Enter` to clone → `n` to name → `c` to commit → `Shift + P` to verify → `y` to push.
Proceeding step-by-step with the TUI reduces the risk of errors or omissions and cognitive load.

## Safety Features

- Only "your own owner repositories" that are "public / non-fork / non-archived" are displayed as clone sources.
- Remotes are reconfigured to a safe state immediately after cloning.
- `upstream` is deleted, and `origin`'s fetch URL is set to HTTPS, push URL to SSH.
- A hard guard is performed before pushing.
- Pushes are rejected if `origin` points to the original norenwake source.
- Pushes are rejected if `upstream` still exists.

## Requirements

- Rust (`cargo`)
- `git`
- `gh` (GitHub CLI)
- `delta` (diff tool)

`gh` is used for API authentication. Tokens are retrieved in the following priority:

1.  `GH_TOKEN`
2.  `GITHUB_TOKEN`
3.  `gh auth token`

## Installation

```bash
cargo install --force --git https://github.com/cat2151/norenwake
```

## Launch

```bash
norenwake
```

## Update

```bash
norenwake update
```

## Key Bindings

- `h` / `l` / `←` / `→`: Move focus pane (repos / dir tree / log)
- `j` / `k` / `↑` / `↓`: Move within current pane
- `PageUp` / `PageDown`: Page movement within current pane
- `Enter` (on repos pane): Clone selected repo
- `n`: Edit new repo name
- `c`: Commit
- `Shift + P`: Pre-push verification and push confirmation
- `Shift + L`: Copy full log to clipboard
- `/`: Open repos filter overlay (space-separated AND search)
- `?`: Help overlay
- `q`: Quit

## Workflow

1.  Select target in `repos` and `Enter` to clone
2.  Edit and confirm `new repo name` with `n`
3.  Commit with `c`
4.  Confirm verification results with `Shift + P`
5.  Press `y` to push

## README Update Rules

`update_readme_ja` maintains only one of the following norenwake headers at the beginning of `README.ja.md`.

```md
# <new repo name>

Cloned from the original repo and branched off. Contains history up to the branching point.
```

If existing headers of the same type are consecutive, they are collapsed to avoid duplication.

## README preview

- Retrieval prioritizes `README.ja.md`, falling back to `README.md` on failure.

## Data Storage Location (Windows)

- `%LOCALAPPDATA%\norenwake\`

## Assumption
- This application is for personal use and is not intended for others. If you want similar functionality, we recommend creating your own.

## What This App Aims For
- PoC. To demonstrate (and has demonstrated) that useful personal applications can be created with Codex.

## What This App Does NOT Aim For (Out of Scope)
- Support. Responding to requests or suggestions.