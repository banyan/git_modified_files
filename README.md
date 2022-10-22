# git_modified_files

>A Git subcommand to list modified files in git commit or from current working tree.

This CLI tool was created with the motivation to easily edit files that are currently being modified or have been modified in the recent commit, and is intended to be used in conjunction with [fzf](https://github.com/junegunn/fzf), [tig](https://github.com/jonas/tig), etc.

## Installation

```
❯ deno install --allow-run -n git_modified_files https://deno.land/x/git_modified_files@0.1.1/main.ts
```

## Usage

Get modified files from current working tree if no argument is given

```
❯ touch a.txt
❯ git_modified_files
a.txt
```

Get modified files in git commit if object (sha1) is given

```
❯ git_modified_files fbb1185
a.txt
```

## License

MIT
