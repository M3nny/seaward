# 🌊 seaward
![crates.io](https://img.shields.io/crates/v/seaward.svg)

## Installation
``` bash
cargo install seaward
```

On NetBSD a pre-compiled binary is available from the official repositories. To install it, simply run:
``` bash
pkgin install seaward
```

An Alpine Linux package is also available in the `community` repository:
``` bash
apk add seaward
```

## Overview
Seaward is a concurrent crawler used for searching word matches in a website, think of it as grep for the web.

### Usage
Run `seaward -h` to show the available options and get started.

To search for a case insensitive word/phrase in a website, use:
```bash
seaward <url> -w <word> -i
```

### Examples
![Example_crawl](assets/seaward.gif)
