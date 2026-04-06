# norenwake

A TUI for Windows that lets you "norenwake" (spin off) your own public GitHub repositories into new, independent ones. Written in Rust.

## Past Challenges and This App's Solutions

| Past Challenges                                                                      | This App's Solutions                                                                                                                                                                                               |
| :----------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| High cognitive load and fatigue from a series of tasks.                              | Visualizes with a TUI, reducing cognitive load.                                                                                                                                                                    |
| Time-consuming to find the target repository.                                        | Quickly find the target repository using `/` for filtering (space-separated AND search).                                                                                                                           |
| Concerns about accidental pushes due to `remote` still pointing to the original repository after cloning. | Reconfigures `remote` immediately after cloning, deletes `upstream`, and standardizes `origin`'s fetch to HTTPS and push to SSH.                                                                                 |
| Settings and README can easily become inconsistent when changing to a new repository name. | Changing the repository name with `n` updates the working directory name, remote settings, and the first header of `README.ja.md` all at once.                                                                  |
| Time-consuming to confirm changes.                                                   | View README preview and diff (delta) within the screen, and logs are preserved.                                                                                                                                    |
| Difficult to notice configuration errors before pushing.                             | Open the verification screen with `Shift + P` to check `origin` / push URL / `upstream`. If in a dangerous state, a hard guard will reject the push.                                                              |

The shortest procedure for practical use is: `Enter` to clone → `n` to name → `c` to commit → `Shift + P` to verify → `y` to push.
By proceeding step-by-step with the TUI, the risk of errors or omissions and cognitive load are reduced.

## Safety Features

-   Only "your own owner repositories" that are "public / non-fork / non-archived" are displayed as clone sources.
-   Reconfigures the `remote` to a safe state immediately after cloning.
-   Deletes `upstream`, sets `origin`'s fetch URL to HTTPS, and push URL to SSH.
-   Implements a hard guard before pushing.
-   Rejects push if `origin` still points to the original 'norenwake' source.
-   Rejects push if `upstream` still exists.

## Requirements

-   Rust (`cargo`)
-   Python
-   `git`
-   `gh` (GitHub CLI)
-   `delta` (diff tool)

Python is required only for the `norenwake update` self-update flow, which delegates to `cat-self-update-lib`.

`gh` is used for API authentication. The token is obtained in the following priority order:

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

## Check for Updates

```bash
norenwake check
```

## Keybindings

-   `h` / `l` / `←` / `→`: Move focus pane (repos / dir tree / log)
-   `j` / `k` / `↑` / `↓`: Move within current pane
-   `PageUp` / `PageDown`: Page movement within current pane
-   `Enter` (on repos pane): Clone selected repo
-   `n`: Edit new repo name
-   `c`: Commit
-   `Shift + P`: Pre-push verification and push confirm
-   `Shift + L`: Copy full log to clipboard
-   `/`: Open repos filter overlay (space-separated AND search)
-   `?`: Help overlay
-   `q`: Quit

## Workflow

1.  Select target in `repos` and `Enter` to clone
2.  Edit and confirm `new repo name` with `n`
3.  Commit with `c`
4.  Check verification results with `Shift + P`
5.  Press `y` to push

## README Update Rules

`update_readme_ja` maintains only one of the following 'norenwake' headers at the beginning of `README.ja.md`:

```md
# <new repo name>

Cloned and branched from the original repo. It holds history up to the branching point.
```

If existing headers of the same type are consecutive, they are folded, and no duplicates are left.

## README preview

-   Retrieval prioritizes `README.ja.md`, falling back to `README.md` on failure.

## Data Storage Location (Windows)

-   `%LOCALAPPDATA%\norenwake\`

## Assumptions
-   This application is intended for personal use and is not designed for others. If you desire similar functionality, we recommend cloning or creating your own.

## What This App Aims For
-   PoC. Demonstrating (and demonstrated) that useful personal applications can be created with Codex.

## What This App Does Not Aim For (Out of Scope)
-   Support. Responding to requests or suggestions.
