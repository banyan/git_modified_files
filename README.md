# git_modified_files

>A Git subcommand to list modified files in git commit or from current working tree.

This CLI tool was created with the motivation to easily edit files that are currently being modified or have been modified in the recent commit, and is intended to be used in conjunction with [fzf](https://github.com/junegunn/fzf), [tig](https://github.com/jonas/tig), etc.

## Installation

```
❯ deno install --allow-run -n git-modified_files https://deno.land/x/git_modified_files@0.1.4/main.ts
```

NOTE: Git has a convenient mechanism that allows users to create their own subcommands by simply placing an executable file called git-xxx, therefore it uses `git-modified_files` instead of `git_modified_files`.

## Usage

Get modified files from current working tree if no argument is given

```
❯ touch a.txt
❯ git modified_files
a.txt
```

Get modified files in git commit if object (sha1) is given

```
❯ git modified_files fbb1185
a.txt
```

## License

MIT
