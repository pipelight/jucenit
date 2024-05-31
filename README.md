# Jucenit - Easy uncompriomising web server.

Warning: Early development stage.

Jucenit is a is a set of utilities to manage
[nginx unit](https://github.com/nginx/unit) web server easily
through scattered toml files.

## Features

- **Split** your configuration across multiple files in **Toml**.
- **Automatic ssl** renewal.

## Usage

Update the global configuration with a configuration chunk like beneath.

```sh
jucenit push
# or
jucenit push --file jucenit.dev.toml
```

Launch a daemon that renews certificats when needed.

```sh
jucenit ssl --watch
```

Or renew certificates immediatly.

```sh
jucenit ssl --renew
```

## Example (reverse-proxy)

At your project route create a `jucenit.toml` file.

```toml
# jucenit.toml

[[unit]]
listeners = ["*:443"]

[unit.match]
host = "example.com"

[unit.action]
proxy = "http://127.0.0.1:8888"
```

This file defines

- a bound to port 443 of localhost public ip.
- that redirects requests to "example.com" to the port 8888 of localhost default private ip.

## Example (file sharing)

At your project route create a `jucenit.toml` file.

```toml
# jucenit.toml

[[unit]]
listeners = ["*:443"]

[unit.match]
host = "test.com"
uri = "/files"

[unit.action]
share =[
    "/path/to/my_files"
]
```

This file defines

- a bound to port 443 of localhost public ip.
- that serves files at "/path/to/my_files" when "test.com/files" is requested.

## How it works ?

Jucenit translates a simpler syntax to the original nginx-unit json.
Every configuration files generate a configuration chunk
that is merged with the existing nginx-unit configuration.

## Install

You need a running instance of [nginx unit](https://github.com/nginx/unit) that listens on port 8080.
Everything can be found in the official documentation.

Create or Modify the systemd/initd unit to run unit bound to the port 8080.

### Nixos

With nixos, install the flake.
and add a nix configuration like what is inside `flake.example.nix`
to your actual nixos configuration.

### Cargo

Install with cargo rust.
