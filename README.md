# cc - CodeCrafters-style practice, fully local

You write the program in any language. `cc` shows what each stage asks for
and runs the **official tester** against your code. No website, no starters,
no solutions.

## Requirements

- Rust 1.70+ (for building `cc`)
- git
- Go (to build the tester) **or** Docker (builds it in a `golang` container)
- Whatever your chosen course's tester needs at runtime. For Redis, the
  tester repo installs the real `redis` package in its own setup/CI, so
  install `redis-cli` (and `redis-server`) on your machine.

## Install

```bash
# Build from source
cargo build --release
sudo cp target/release/craftcoders /usr/local/bin/cc
# or add target/release to your PATH
```

Or run directly with `cargo run -- <command>`.

## Use

```bash
cc init redis my-redis     # clones course + tester, builds tester, makes the folder
cd my-redis
cc show                    # instructions for the current stage
# ... write your program; edit your_program.sh so it starts it ...
cc test                    # runs all earlier stages + the current one
cc submit                  # same, and moves to the next stage if everything passes
cc status                  # progress
cc ext list                # extensions (Lists, Streams, Transactions, ...)
cc ext add lists           # append that extension's stages after the base stages
cc update                  # pull latest course + tester and rebuild
```

Other courses work the same way: `cc init git`, `cc init sqlite`, `cc init shell`.
It assumes the repos `build-your-own-<course>` and `<course>-tester`; if a
tester is named differently, pass `--tester-repo NAME`.

### Additional commands

```bash
cc doctor                  # check environment (git, go, docker, runtime deps)
cc list                    # list available courses
cc open                    # open current stage markdown in $EDITOR
cc show 3                  # show specific stage (by number or slug)
cc show --raw              # raw markdown without terminal rendering
cc show --json             # JSON output for scripting
cc test --only-current     # run only current stage (skip previous)
cc test --upto 5           # run up to stage 5
cc test --docker python:3.12  # run tester in Docker container
cc submit --docker ubuntu:22.04 # submit using Docker
```

## Entry point

The tester starts your program by running `your_program.sh` in the project
root (`cc init` creates a placeholder). Point it at your implementation:

```sh
#!/bin/sh
exec python3 -u app/main.py "$@"
```

Set `debug: true` in `codecrafters.yml` for verbose tester logs.

## Running the tester in Docker

By default the tester runs on your machine. To run it in a container instead
(your project mounted at `/app`):

```bash
cc test --docker python:3.12
```

The image must contain your language toolchain (and `redis-cli` for Redis).
The tester binary is mounted in, so it has to be a Linux build: that's what
you get if you build on Linux, or with `cc init` falling back to Docker.

## Where things live

- `~/.cc/courses/<course>/definition` - course repo (stage list + markdown)
- `~/.cc/courses/<course>/tester` - tester source, binary in `../bin/tester`
- `<project>/.cc.json` - your progress (completed stage slugs, extensions)

Set `CC_HOME` to move the first two.

## Notes

- Stage text comes from the course repo's `stage_descriptions/`. The site's
  template blocks are resolved for a reader with no fixed language:
  bot-only and single-language blocks are dropped.
- Course content and testers are MIT-licensed public repos; this tool only
  reads them.
- Colors can be disabled with `--no-color` or the `NO_COLOR` environment variable.