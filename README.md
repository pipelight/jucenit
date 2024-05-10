# Jucenit - Easy proxy with autossl

Jucenit is a is a small cli built with **Rust** on top of
[nginx unit](https://github.com/nginx/unit).

Warning: Early development stage.

## Usage

Nginx unit works with a configuration API and without any main config file.
Jucenit eases interactions with unit through toml/yaml files. It is much like
caddy but with the ability to split config in multiple files.

Let's say you want to put your documentation website online.

At your project route create a `jucenit.toml` file.

This file generates a configuration chunk that is merged with the existing
configuration.

```toml
[[unit]]
listeners = ["*:80","*:443"]

[unit.match]
uri = "http://example.com/"

[unit.action]
proxy = "http://127.0.0.1:8888"
```

To see the resulting unit configuration.

```sh
jucenit adapt
```

Edit the whole configuration with your favorite editor

```sh
jucenit edit
```
