# Crossmarks
A simple tool to generate bookmarks keybinds for lf, zsh named dirs and cd aliases from a single source file.

```
crossmarks -h
Usage: crossmarks --input <BOOKMARKS_FILE> <--lf <LF_FILE>|--zsh <ZSH_NAMED_DIRS_FILE>|--cd-alias <CD_ALIASES_FILE>>

Options:
  -i, --input <BOOKMARKS_FILE>
  -l, --lf <LF_FILE>
  -z, --zsh <ZSH_NAMED_DIRS_FILE>
  -c, --cd-alias <CD_ALIASES_FILE>
  -h, --help                        Print help
```

## Input file
Each entry consists a of space separated shortcut and the directory it points to, one entry per line. Lines beggining with '#' are considered comments.

## Example
Input file:
```
d ~/downloads
D ~/desktop
```
will generate following files:
### lf
```
map gd cd ~/downloads
map gD cd ~/desktop
```
### zsh
```
hash -d d=~/downloads
hash -d D=~/desktop
```
### cd
```
alias cdd="~/downloads"
alias cdD="~/desktop"
```
