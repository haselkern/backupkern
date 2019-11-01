# backupkern

A tiny backup program I made for backups of my home folder.

backupkern writes the folder you want to backup to a location of your choice. You can ignore any files you do not want to backup. The backups will be plain folders, so you can easily examine them. To keep the backup small, backupkern will hardlink any files that were not modified between backups.

You are wholeheartedly invited to contribute to this repository by creating pull requests or submitting issues!

## Installation

You need to install [Rust](https://www.rust-lang.org/tools/install) and clone this repository. Run `cargo build --release` and enjoy the binary at `target/release/backupkern` :tada:

## Usage

~~~
backupkern [--config alternative_config.yaml]
~~~

It will use the config in `~/backupkern.yaml` by default. See [backupkern.yaml](https://github.com/haselkern/backupkern/blob/master/backupkern.yaml) for an example config file with comments.

## Issues

-  [ ] Remove `cp` dependency: backupkern relies on the `cp` command to copy file permissions and modification dates. This dependency should be removed to allow cross platform usage.
-  [ ] Ignore patterns: It should be possible to specify regexes, which match absolute paths that should be ignored
-  [ ] Ignore Unix pipes (?): The `cp` command gets stuck when trying to copy a Unix pipe file. You want to check your progress regularly while doing your first backup and add eventual pipe files to the ignore section in your config.
