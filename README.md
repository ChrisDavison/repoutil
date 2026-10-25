# repoutil

Wrapper around my common git commands

---

I have many repositories (in terms of followed, personal, or work).

For this, I want a wrapper that implements common functionality across all of them. I've tried things like [mixu/gr](https://github.com/mixu/gr), but wanted to streamline the process to provide *exactly* the output I want, and hide the rest.

Commands include:

- `fetch` (fetch all remotes, then show the status summary)
- `branchstat` (ahead/behind, modified/untracked counts) — the default when no subcommand is given
- `branches` (local branch names)
- `list` and `add` for managing the config file
- `jj` support (when built with the `jj` feature): branchstat also reports a count of mutable jj commits

## `fetch`

Visit each configured repo in parallel, fetching remote changes for all remotes, then show each repo's status summary.

## `branchstat`

For each configured repo, show ahead/behind against its upstream plus counts of added, deleted, modified, and untracked files. The current branch is shown after the path when it is not `main` or `master`. Repos with nothing to report are hidden.

## Options

- `--color <auto|always|never>`: Control colored output (a non-empty `NO_COLOR` disables colors)
- `--keep-home`: Do not strip the common leading path from displayed repo paths

## Configuration

Populate `~/.repoutilrc` with lines containing either repo paths or directories to scan for repos. Lines starting with `!` are exclusions; blank lines and `#` comments are ignored. `~` is expanded at the start of the path.
