# Jucenit - nginx unit wrapper

A wrapper around **nginx unit** to ease queries, ssl support and container
proxying.

Better to use in combination with
[pipelight](https://github.com/pipelight/pipelight)

## Motivations

Nginx unit is an amazing piece of software that allow us to configure a web
server programmatically.

It is so easy to configure unit that there is actually a lake of open-source
solutions for easy cli configuration besides the official cli
[unitd](https://github.com/nginx/unit/tools/unitc)

If you don't have a custom solution yet, Jucenit got you cover with:

- easy ssl renewal,
- easy container proxying,

## Prerequisites

Install dpendencies: nginx-unit, deno, openssl, certbot.

## Usage as cli

Use it in command line.

Query unitd for its configuration .

### Getters

Get the whole configuration

```sh
jucenit info
```

Get certificates

```sh
jucenit certs
```

more on the help menu.

### Setters

Containers optimized Create a listener, routes and certificates

```sh
jucenit domain '{ name: "container_name", port: { out: 8084 }}'
```

## Usage with pipelight

[Pipelight](https://github.com/pipelight/pipelight) is a cli to run pipelines
defined in Typescript.

You can use it to avoid typing your full container definition on the command
line. Define your container with pipelight:

```ts
// pipelight.ts
const docker = new Docker({
  globals: {
    dns: "example.com",
    version: "dev",
  },
  containers: [{
    suffix: "front",
    ports: [{
      in: 80,
      out: 8081,
    }],
  }],
});
const container = docker.containers.get("front") as Container;
```

And expose container:

In the background it generates an ssl certificate with standalon certbot and add
the specific configuration in unitd.

```ts
// pipelight.ts
const container = docker.containers.get("front") as Container;
let expose = step(() => [
  // pass container definition
  `jucenit domain ${container}`,
]);
```

## Connect to Unit

By default, jucenit attenpts to connect to a nginx\_unit server at
http://127.0.0.1:8080.

Connect to a custom url with custom url with:

```sh
jucenit --url <custom_url> --socket <socket> info
```

## Contribute

```sh
deno test
```
