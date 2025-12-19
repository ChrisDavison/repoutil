% repoutil(1) Run common operations on multiple repos
%
% 2021-03-21


# NAME

repoutil - utility for multiple git repos

# SYNOPSIS

    repoutil list [--json]
    repoutil add
    repoutil git stat|fetch|pull|push|branchstat|branches|stashcount|unclean|dashboard [--json]
    repoutil jj stat|sync [--json]

# DESCRIPTION

This script relies on a file, *~/.repoutilrc*, which contains either individual
directory paths or a root folder containing multiple sub directories.

**list**
: list repos that would be operated on

**add**
: append the current directory to *~/.repoutilrc*

**git stat**
: run *git status -s -b* for every repo, and show only repos with changes

**git fetch**
: fetch remote changes for all branches

**git pull|push**
: pull or push changes for all branches and tags

**git branchstat**
: list ahead/behind and modified/untracked counts for each repo

**git branches**
: list local branch names for each repo

**git stashcount**
: show the number of stashes for each repo

**git unclean**
: list repos that have local changes

**git dashboard**
: show repo status plus recent commits

**jj stat|sync**
: jujutsu equivalents when built with the `jj` feature

# OPTIONS

**--json**
: emit machine-readable JSON of the form `{ "items": [{ title, arg, subtitle? }] }`

**--color**=auto|always|never
: control colored output; in `auto`, `NO_COLOR` disables colors

**--threads**=N
: cap Rayon parallelism to N threads


# AUTHORS

Chris Davison <c.jr.davison@gmail.com>
