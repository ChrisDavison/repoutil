# repoutil

Wrapper around my common git commands

---

I have many repositories (in terms of followed, personal, or work).

For this, I want a wrapper that implements common functionality across all of them. I've tried things like [mixu/gr](https://github.com/mixu/gr), but wanted to streamline the process to provide *exactly* the output I want, and hide the rest.

Commands include:

- `git fetch`/`git pull`/`git push`
- `git stat` (short repo status) and `git branchstat` (ahead/behind, modified/untracked counts)
- `git branches`, `git stashcount`, `git unclean`, `git dashboard`
- `list` and `add` for managing the config file
- `jj stat` and `jj sync` (when built with the `jj` feature)

## `stat`

For each configured repo, return the `git status -s -b` output. Only show output for repos with interesting changes.

## `fetch`

Visit each configured repo, fetching remote changes for all branches.

Options:

- `--json`: Emit machine-readable JSON: `{ "items": [{ title, arg, subtitle? }] }`
- `--color <auto|always|never>`: Control color output (respects `NO_COLOR` in `auto`)
- `--threads <N>`: Cap parallelism for large repo sets
- `--keep-home`: Do not strip the common home prefix from displayed paths

Configuration:

Populate `~/.repoutilrc` with lines containing either repo paths or directories to scan for repos. Lines starting with `!` are exclusions; blank lines and `#` comments are ignored. `~` is expanded at the start of the path.
