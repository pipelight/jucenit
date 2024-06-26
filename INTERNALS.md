# Jucenit internals

## What it is.

I do not have the time to maintain such a good piece of software
as a mature server like nginx-unit.

Jucenit is only a translation layer from an easy toml syntax
to nginx-unit configuration file.

Plus some utilities to ease ssl renewal and live edit configuration.

## How it works.

The toml configuration is split into entities in a sqlite database.
The entities are then reassembled into the nginx-unit configuration.

```mermaid
graph LR

A(jucenit.toml) ---> B(Sql_Database) ---> C(nginx-unit.json)

```
