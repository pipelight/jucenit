# Jucenit - Nginx Unit wrapper

A wrapper around nginx unit to ease ssl support and container proxying.

Better to use in combination with
[pipelight](https://github.com/pipelight/pipelight)

The main goal of this tool is to expose desired containers as soon as deployed
with pipelight

## Usage

Use it in command line.

```sh
jucenit info
```

```sh
jucenit certs
```

more on the help menu.

## With pipelight

```ts
step(()=>[
    // pass container definition
    `jucenit domain ${container}`
]
```

## Connect to Unit

Attenpt to connect to a nginx\_unit server at http://127.0.0.1:80.

Or add a custom url with:

```sh
jucenit --url <custom_url> --socket <socket> info
```
