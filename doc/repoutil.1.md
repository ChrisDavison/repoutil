% repoutil(1) Run common operations on multiple repos
%
% 2021-03-21


# NAME

repoutil - utility for multiple git repos

# SYNOPSIS

    repoutil [add|list|fetch|branchstat|branches] [--color auto|always|never] [--keep-home]

# DESCRIPTION

This script relies on a file, *~/.repoutilrc*, which contains either individual
directory paths or a root folder containing multiple sub directories.

**add**
: append the current directory to *~/.repoutilrc* (must be a git repo)

**list**
: list repos that would be operated on

**fetch**
: fetch remote changes for all remotes and tags, then show the
per-repo summary below (equivalent to *branchstat* after fetching)

**branchstat**
: list ahead/behind and modified/untracked counts for each repo;
repos with nothing to report are hidden. Run by default when no
subcommand is given

**branches**
: list local branch names for each repo

When built with the `jj` feature, **branchstat** also reports a
per-repo count of mutable jj commits.

# OPTIONS

**--color**=auto|always|never
: control colored output; in `auto`, a non-empty `NO_COLOR`
disables colors

**--keep-home**, **-k**
: do not strip the common leading path from displayed repo paths

# CONFIGURATION

Each line of *~/.repoutilrc* is either a repo path or a directory to
scan for repos. Lines starting with `!` exclude a path, and blank
lines and `#` comments are ignored.

# AUTHORS

Chris Davison <c.jr.davison@gmail.com>
