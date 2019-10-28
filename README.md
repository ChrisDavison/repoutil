# repoutil

Wrapper around my common git commands

---

I have many repositories (in terms of followed, personal, or work).

For this, I want a wrapper that implements common functionality across all of them. I've tried things like [mixu/gr](https://github.com/mixu/gr), but wanted to streamline the process to provide *exactly* the output I want, and hide the rest.

As such, currently only `stat` and `fetch` are listed as commands.

## `stat`

For each subdirectory of `$CODEDIR`, return the `git status -s -b` output. Only show output of repos that have anything interesting to say.

## `fetch`

Visit each subdirectory of `$CODEDIR`, pulling remote changes for all branches.
