# Running Migrator CLI

Create the required database schema (tables)

```sh
sea-orm-cli migrate fresh
```

Do not use. Entities already exists.

```sh
sea-orm-cli generate entity --output-dir ./entity/src
```
