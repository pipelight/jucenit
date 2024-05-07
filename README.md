# Jucenit - Easy server web with autossl

Jucenit is a is a small cli built with **Rust** on top of
[nginx unit](https://github.com/nginx/unit).

## Usage

Create a `jucenit.toml` file.

```toml
[[unit]]
listeners = ["*:80","*:443"]

[unit.match]
uri = "http://example.com/"

[unit.action]
proxy = "http://127.0.0.1:8888"
```
