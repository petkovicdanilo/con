# con
Simple program to run Linux containers written in Rust.

## Requirements

- fairly recent Linux kernel
- cgroups v1
- `newuidmap` and `newgidmap` programs

In order to use `con` as non-root user, you need to
set up cgroups as root user. Run (only once and on computer restart):
```bash
$ sudo chmod +x init.sh
$ sudo ./init.sh
```

## Features

- rootless containers
- volumes
- environment variables
- download images from Docker registry
- resource limiting using cgroups

## Usage

Command `con --help` will list you all options:
- `pull` - pulling the image
- `run` - creating container from image (pulling it if it does not exist on disk)
and running it

[![asciicast](https://asciinema.org/a/445035.svg)](https://asciinema.org/a/445035)

## Next steps

- networking
- support for building images
  - Dockerfile build
- detached containers
- docker `exec` functionality