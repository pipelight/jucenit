# Jucenit - A simple web server.

Warning: Early development stage.

Jucenit is a web server configurable through short scattered toml files.
Internally uses [nginx unit](https://github.com/nginx/unit).

## Features

- **Split** your configuration across multiple files in **Toml**.
- **Easy ssl** renewal.

## How it works ?

See INTERNALS.md

## Usage

### Expose services

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

Or for file sharing

```toml
# jucenit.toml
[[unit]]
uuid = "f37490cb-d4eb-4f37-bb85-d39dad6a21ab"
listeners = ["*:443"]
[unit.match]
hosts = ["test.com"]
uri = "/files"
[unit.action]
share = ["/path/to/my_files"]
```

And many more possibilities at [nginx unit](https://github.com/nginx/unit).
Update the global configuration with your configuration chunks.

```sh
jucenit push
# or
jucenit push --file jucenit.toml
```

**Your chunks must be uniquely identified with a mandatory uuid.**

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

## Install

You first need a running instance of nginx-unit.
See the [installation guide](https://unit.nginx.org/installation/):

Add the following configuration changes:

```sh
unitd --control '127.0.0.1:8080'
```

So it listens on tcp port 8080 instead of default unix socket.

### with Cargo

Install on any linux distribution with cargo.

```sh
cargo install --git https://github.com/pipelight/jucenit
```

### with Nixos

Install on Nixos.

First, add the flake url to your flakes **inputs**.

```nix
inputs = {
    jucenit.url = "github:pipelight/jucenit";
};
```

Then add a nix configuration like what is inside `flake.example.nix`
to your actual nixos configuration.

## Roadmap

Tooling:

Convenience commands:

- [x]: edit global configuration with favorite editor
- [ ]: "push -d" to remove a chunk from global configuration
- [ ]: allow passing a string to "jucenit push"

Tls/Ssl:

ACME Challenge support:

- [x]: http-01
- [ ]: tls-ALPN-01

Automatic certificate renewal:

- [ ]: parallel certificate renewal
- [ ]: make a daemon that watches certificates validity
- [ ]: provide a template systemd unit (nginx sandboxing oc)
