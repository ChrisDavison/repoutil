% repoutil(1) Run common operations on multiple repos
%
% 2021-03-21


# NAME

repoutil - utility for multiple git repos

# SYNOPSIS

    repoutil stat
    repoutil fetch
    repoutil list
    repoutil unclean
    repoutil branchstat
    repoutil branches

# DESCRIPTION

This script relies on a file, *~/.repoutilrc*, which contains either individual
directory paths or a root folder containing multiple sub directories.

**stat**
: run *git status* for every repo

**fetch**
: get remote changes for every branch

**list**
: list repos that would be operated on

**unclean**
: list repos that have local changes

**branchstat**
: list short status of every branch for each repo

**branches**
: list branch names for each repo


# AUTHORS

Chris Davison <c.jr.davison@gmail.com>
