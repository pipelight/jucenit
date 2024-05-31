# Jucenit - A simple web server.

Warning: Early development stage.

Jucenit is a web server configurable through short scattered toml files.
Internally uses [nginx unit](https://github.com/nginx/unit).

## Features

- **Split** your configuration across multiple files in **Toml**.
- **Automatic ssl** renewal.

## Usage

### Expose services

Use it as a reverse-proxy.

```toml
# jucenit.toml
[[unit]]
listeners = ["*:443"]

[unit.match]
host = "example.com"

[unit.action]
proxy = "http://127.0.0.1:8888"
```

Or for file sharing

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

And many more possibilities at [nginx unit](https://github.com/nginx/unit).

Update the global configuration with your configuration chunks.

```sh
jucenit push
# or
jucenit push --file jucenit.toml
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

## How it works ?

Jucenit only convert files.

Jucenit translates its simple **toml** configuration into nginx-unit **json** configuration.
It then pushes those file chunks through nginx-unit API.

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
