# git_modified_files

>A Git subcommand to list modified files in git commit or from current working tree.

This CLI tool was created with the motivation to easily edit files that are currently being modified or have been modified in the recent commit, and is intended to be used in conjunction with [fzf](https://github.com/junegunn/fzf), [tig](https://github.com/jonas/tig), etc.

## Installation

Requires [Rust](https://rustup.rs/).

```
❯ cargo install --git https://github.com/banyan/git_modified_files
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

## Development

```
❯ cargo test
```

NOTE: Up to 0.1.4 this was a Deno module published on deno.land/x. Deno's development ends in 2027, so 0.2.0 is a Rust rewrite with the same output (paths with spaces and `.woff`/`.woff2` files are now handled correctly).

## License

MIT
