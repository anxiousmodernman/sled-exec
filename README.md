# sled-exec

Command line wrapper that captures stdout and stderr in a sled embedded database.

## Usage

Simply pass a subcommand and its args after `--`. For example

```
sled-exec -- echo "hello world"
```

A database named `sled-exec.db` will be created in the current directory by default.
