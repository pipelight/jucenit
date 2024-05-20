# Jucenit - Easy uncompriomising web server.

Warning: Early development stage.

Jucenit is a is a set of utilities to manage
[nginx unit](https://github.com/nginx/unit) web server easily
through scattered toml/yaml files.

## Features

- **Split** your configuration across multiple files in **Toml and yaml**.
- **Automatic ssl** renewal.

## Example (reverse-proxy)

At your project route create a `jucenit.toml` file.

It binds to port 80 and 443 of public localhost
and redirects every requests to "example.com" to the port 8888.

```toml
# jucenit.example.toml

[[unit]]
listeners = ["*:80","*:443"]

[unit.match]
host = "example.com"

[unit.action]
proxy = "http://127.0.0.1:8888"
```

## Usage

Update global configuration with a configuration chunk like above.

_Either seeks for the nearest `jucenit*.toml` file_

```sh
jucenit push
```

_or provide a file path_

```sh
jucenit push --file
```

Convert jucenit toml configuration into nginx-unit json configuration.
(S/O `caddy adapt` command).

```sh
jucenit adapt --file jucenit.example.toml
```

```json
{
  "listeners": {
    "*:80": {
      "pass": "routes/jucenit_[*:80]"
    },
    "*:443": {
      "pass": "routes/jucenit_[*:443]"
    }
  },
  "routes": {
    "jucenit_[*:443]": [
      {
        "action": {
          "proxy": "http://127.0.0.1:8888"
        },
        "match": {
          "uri": "http://example.com/"
        }
      }
    ],
    "jucenit_[*:80]": [
      {
        "action": {
          "proxy": "http://127.0.0.1:8888"
        },
        "match": {
          "uri": "http://example.com/"
        }
      }
    ]
  }
}
```

Edit the whole configuration with your favorite editor.
It is actually the only way to delete configuration parts.

```sh
jucenit edit
```

## How it works ?

Common things we do with a web server is proxying and load-balancing.
Jucenit focuses those use cases.

Jucenit translates a simpler syntax to the original nginx-unit json.
Every configuration files generate a configuration chunk that is merged with the existing
ray nginx-unit configuration.

### Flexibility

If you seek complexity and wish to tear appart your web server
you can use jucenit simple syntax aside of raw nginx-unit Json configuation.
No need to use multiple web servers.

## Install

You need a running instance of [nginx unit](https://github.com/nginx/unit) that listens on port 8080.
Everything can be found in the official documentation.

Create or Modifi the systemd/initd unit to run unit bound to the port 8080.

### Nixos

With nixos, install the flake.
and add a nix configuration like what is inside `flake.example.nix`
to your actual nixos configuration.

### Cargo

Install with cargo rust.
