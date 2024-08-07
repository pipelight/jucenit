# Jucenit - A simple web server.

Warning:

**Early development stage.**
Do not use at home.
You might not want to come back to other web servers.

The API is still undergoing some small changes.

Jucenit is a web server configurable through short scattered toml files.
Internally uses [nginx unit](https://github.com/nginx/unit).

## Features

- **Split** your configuration across multiple files in **Toml**.
- **Easy ssl** renewal.

## Usage

### Expose services

**Your configuration chunks must be uniquely identified with a mandatory uuid.**

Use it as a reverse-proxy.

```toml
# jucenit.toml
[[unit]]
uuid = "d3630938-5851-43ab-a523-84e0c6af9eb1"
listeners = ["*:443"]
[unit.match]
hosts = ["example.com"]
[unit.action]
proxy = "http://127.0.0.1:8888"
```

On queries like "https://example.com"
it redirects to the port 8888 on private network.

Or for file sharing

```toml
# jucenit.toml
[[unit]]
uuid = "f37490cb-d4eb-4f37-bb85-d39dad6a21ab"
listeners = ["*:443"]
[unit.match]
hosts = ["test.com"]
uri = "/static"
[unit.action]
share = ["/home/website/static"]
```

On queries like "https://test.com/static/index.html"
it redirects to /home/website/static/index.html

And many more possibilities at [nginx unit](https://github.com/nginx/unit).
Update the global configuration with your configuration chunks.

```sh
jucenit push
# or
jucenit push --file jucenit.toml
```

### Edit the global configuration

The only way to cherry remove chunks from the global configuration
is to edit the main configuration with:

```sh
jucenit edit
```

Or to delete everything previously pushed to the global configuration

```sh
jucenit clean
```

### Tls/Ssl management

Add new certificates or Renew almost expired certificates.

```sh
jucenit ssl --renew
```

Remove every certificates.

```sh
jucenit ssl --clean
```

Run the daemon for automatic certificate creation and renewal

```sh
jucenit ssl --watch
```

## How it works ?

See detailed project structure and functionning at [INTERNALS.md](https://github.com/pipelight/jucenit/INTERNALS.md)

## Install

### with Nix and Nixos

First, add the flake url to your flakes **inputs**.

```nix
inputs = {
    jucenit.url = "github:pipelight/jucenit";
};
```

And enable the service in your configuration file;

```nix
services.jucenit.enable = true;
```

### with Cargo

You first need a running instance of nginx-unit.
See the [installation guide](https://unit.nginx.org/installation/):

Add the following configuration changes:

```sh
unitd --control '127.0.0.1:8080'
```

So it listens on tcp port 8080 instead of default unix socket.

Install on any linux distribution with cargo.

```sh
cargo install --git https://github.com/pipelight/jucenit
```

You need to run a background deamon for autossl.

Create a file like a systemd-unit file or an initd file
for autossl.

It must run the following command:

```sh
jucenit ssl --watch
```

## Roadmap

cli:

- [x] add command to edit global configuration with favorite editor.
- [x] add option to allow passing a toml string instead of a config file path to the executable.
- [ ] add "push -d" to remove a chunk from global configuration.

ssl certificates:

- [x] parallel certificate renewal
- [x] provide a template systemd unit (with nginx-unit sandboxing of course)
- [x] add support for acme challenge http-01
- [ ] add support for acme challenge tls-ALPN-01

automation:

- [x] make a daemon that watches certificates validity

global improvements:

- [ ] add a verbosity flag and better tracing

## Authors note

_We need better tooling to easily share our makings to the world._

Licensed under GNU GPLv2 Copyright (C) 2023 Areskul
