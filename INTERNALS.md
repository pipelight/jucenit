# Jucenit internals

## What it is.

I do not have the time to maintain such a good piece of software
as a mature server like nginx-unit.

Jucenit is only a translation layer from an easy toml syntax
to nginx-unit configuration file.

Plus some utilities to ease ssl renewal and live edit configuration.

## Crate structure

Database crates:

- migration: Database schema definition.
- entity: Autogenerated with ORM.

Convenience crates:

- jucenit: Entrypoint for binary.
- utils: Pipelight utility crate for filesystem and process management.

Core crate:

- jucenit_core: The main crate were the everything is done.
  Core modules:
  - cast: toml to database entities
  - nginx: database entities to json and ssl renewal.

## How it works.

### File type convertion.

The toml configuration is split into small related entities that are pushed to a relational database (sqlite).
The entities are then taken from the database,
and reassembled into the equivalent nginx-unit configuration.

Yes, the database serves the solo purpose of file convertion,
so it is has a pretty simple schema, but it drastically decreases the code comlexity.

```mermaid
graph LR

A(jucenit.toml) ---> B{Database} ---> C(nginx-unit.json)

```

The database schema is defined inside the **migration crate** through a practical rust orm
[sea_orm](https://www.sea-ql.org/SeaORM/docs/index/).

_I have been traumatised with ORM so I could have written raw SQL, but SeaORM really
does a pleasant heavy lifting._

Simplified diagram without relation tables.

```mermaid
classDiagram
    Match <|-- Action
    Match <|-- Host
    Match <|-- Listener

    class Match {
        raw_parameters
    }
    class Host {
        raw_parameters
    }
    class Listener {
        raw_parameters
    }
    class Action {
        raw_parameters
    }

```

Complete diagram with relation tables.

Every relations are many to many through a relation table.

```mermaid
classDiagram
    Match <|-- MatchListener
    MatchListener <|-- Listener

    Match <|-- MatchHost
    MatchHost <|-- Host

    Match <|-- Action

    class Host {
        +int id
        +String domain
    }
    class Match {
        +int id
        +int uuid
        +int action_id
        +String raw_params
    }
     class Listener {
        +int id
        +String raw_params
    }
    class Action {
        +int id
        +String raw_params
    }
    class MatchHost {
        +int id_match
        +int id_host
    }
    class MatchListener {
        +int id_match
        +int id_listener
    }
```

The column `raw_params` are the nginx-unit arguments stored as json.

Because nginx-unit and jucenit will evolve for the better, and for the sake of simplicity,
their is no strong mapping between jucenit and nginx through clearly defined rust Structs(type definitions).
Consequences are that jucenit will always accept arguments that are accepted by nginx-unit
without the need to update jucenit internals Structs.

### Auto Ssl (tls certificate management)

Relate on a slighty modified version of [acme2](https://docs.rs/acme2/latest/acme2/) crate,
which is [pipelight-acme2](https://github.com/pipelight/acme2).
