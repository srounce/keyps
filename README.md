# Keyps

**Key** **P**rovisioning **S**ervice

_Provision `authorized_keys` from HTTPS/GitHub/GitLab and automatically keep them up to date._

## Motivation

### Problem

Provisioning the `authorized_keys` for a user is usually either a cumbersome process, requiring a human to manually update a list of keys and redeploy every time a key changes or a person joins/leaves the team, or it involves setting up secret management providers that can be overkill in many situations. The needs of many sysadmins lie between these two extremes.

### Solution

`keyps` aims to fill part of that gap by simplifying and automating the provisioning of `authorized_keys` by (re-)using infrastructure/services that are already ubiquitous. This allows individual team members to manage their keys and have those changes reflected on the machines they've been permitted access to without redeploying or deploying complicated additional infrastructure.

## Installation

- Nix: `nix run github:srounce/keyps`

<span style="opacity:.25">**TODO: Improve this section**</span>

## Usage

### Example

```bash
$ keyps -s github:srounce
```

### Options

- `-v...`

  Verbosity level, the more `v`s the more verbose program output will be.

  _Example:_ `-vvv`

- `-f`, `--file <FILE>`

  Path to authorized_keys file (eg. ./authorized_keys). This file must exist and be writable.

  If not specified, an upward search for the closest available `.ssh/authorized_keys` file will be performed from the current working directory.

- `-s`, `--source <SOURCES>`

  One or more sources with one of the following formats:

  - `github:<username>`
  - `gitlab:<username>`
  - `https://example.com/my.keys`

- `-i`, `--interval <INTERVAL>`

  Time in seconds to wait between polling sources

  _Default: 10_

- `-h`, `--help`

  Print help (see a summary with '-h')

- `-V`, `--version`

  Print version
