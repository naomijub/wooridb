# WooriDB
[USER GUIDE](https://naomijub.github.io/wooridb/)

WooriDB is a general purpose (**EXPERIMENTAL**) time serial database, which means it contains all entities registries indexed by DateTime. It is schemaless, key-value storage and uses its own query syntax that is similar to SparQL and Crux's Datalog. 

Some other features are:
- Hashing keys content with [`ENCRYPT`](https://github.com/naomijub/wooridb#create-entity) keyword.
- Hashed values are filtered out and can only be checked with  [`CHECK`](https://github.com/naomijub/wooridb#checks-validity-of-of-an-encrypted-key) keyword.
- [`Ron`](https://github.com/ron-rs/ron/blob/master/docs/grammar.md) schemas for input and output.
  - [ ] JSON to be supported via feature.
  - [ ] EDN to be supported via feature.
- Entities are indexed by `entity_name` (Entity Tree), `DateTime` (Time Serial) and `Uuid` (Entity ID). Entity format is a HashMap where keys are strings and values are supported [`Types`](https://github.com/naomijub/wooridb/blob/main/wql/src/lib.rs#L78).
- Stores persistent data locally.
  - [ ] `S3` as a backend is to be developed.
  - [ ] `Postgres` as a backend is to be developed.
  - [ ] `DynamoDB` as a backend is to be developed.
- Able to handle very large numbers when using the `P` suffix.
  - Ex: `98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P`.
- Configuration is done via environment variables.
  - [ ] Non sensitive configurations are done with `Config.toml`.
  - [ ] CORS
- Authentication and Authorization via session token
  - [ ] Creating and removing ADMINs/new users.
- [Conditional Update](https://github.com/naomijub/wooridb#match-update-entity)
- [ ] Possible Relation Algebra
- [x] Entity history

`Woori` means `our` and although I developed this DB initially alone, it is in my culture to call everything that is done for our community and by our community **ours**.

This project is hugely inspired by:
- [Crux](https://github.com/juxt/crux); 
- [Datomic](https://www.datomic.com/); 
- [Prometheus](https://github.com/prometheus/prometheus) 
- [SparQL](https://en.wikipedia.org/wiki/SPARQL).
- [Database Internals](https://www.amazon.com.br/Database-Internals-Alex-Petrov/dp/1492040347/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Database+Internals%3A&qid=1612831621&sr=8-1)
- [Database System Concept](https://www.amazon.com.br/dp/B073MPV4YC/ref=dp-kindle-redirect?_encoding=UTF8&btkr=1)
- [Designing Data Intensive Application](https://www.amazon.com.br/Designing-Data-Intensive-Applications-Reliable-Maintainable-ebook/dp/B06XPJML5D/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Designing+Data%E2%80%93Intensive+Applications&qid=1612831724&s=books&sr=1-1)
- Professor [Andy Pavlo](http://www.cs.cmu.edu/~pavlo/) Database classes. 
- [Zero Trust in Time Series Data?](https://www.ockam.io/learn/blog/trust_influxdb)


## Installation

To run WooriDB it is necessary to have Rust installed in the machine. There are two ways to do this:

1. Go to rustup.rs and copy the command there, for unix it is `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. Clone WooriDB and execute `make setup`.


### Executing WooriDB

- `Release mode performance`: `make release` in project root for performance optimization.
- `Release mode size`: `make run` in project root for size optimization.
- `Debug mode`: `make debug` in project root.

### Docker
you can find the latest docker image at **[naomijub/wooridb](https://hub.docker.com/repository/docker/naomijubs/wooridb)**. The current most stable tag is **`beta-2`**. To execute the docker container run:

* `docker run -p 1438:1438 naomijubs/wooridb:beta-2 debug` for debug mode.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-2 run`  for size optimization.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-2 release` for performance optimization.
* All `-e/--env` can be replaced by a `--env-file path/to/your/.env`. Your `.env`file should contain the following fields:
```
HASHING_COST=16
PORT=1438
AUTH_HASHING_COST=8
ADMIN=your-admin-id
ADMIN_PASSWORD=your-admin-pswd
``` 

## Usage
* Responses are in [`RON`](https://github.com/ron-rs/ron) format. Support for `JSON` and `EDN` will be done later by using features.
* For now only persistent local memory is used. Support for `S3`, `Postgres` and `DynamoDB` will also be done later by using features.
* **Precise floats** or **numbers larger than f64::MAX/i128::MAX** can be defined with an UPPERCASE `P` at the end. 
  * _Note_: This type cannot be updated with `UPDATE CONTENT`. 
  * Ex.: `INSERT {a: 98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P, } INTO my_entity`.
* `BLOB` will not be supported. Check out [To BLOB or Not To BLOB: Large Object Storage in a Database or a Filesystem](https://www.microsoft.com/en-us/research/publication/to-blob-or-not-to-blob-large-object-storage-in-a-database-or-a-filesystem/).
* To configure hashing cost and port some environment variables are required:
```
HASHING_COST=16
PORT=1438
```

## TODOS
- [ ] Read infos from ztsd files [issue 28](https://github.com/naomijub/wooridb/issues/28)
- [ ] Benchmarks. PRs [61](https://github.com/naomijub/wooridb/pull/61) [93](https://github.com/naomijub/wooridb/pull/93)

## Current Benchmarks
>  MacBook Pro, 2.2 GHz Intel Core i7, 16 GB 2400 MHz DDR4

- `create_entity`           time:   [15.269 ms 15.332 ms 15.396 ms]
- `insert_entity`           time:   [27.438 ms 28.177 ms 28.958 ms]
- `update_set_entity`       time:   [24.110 ms 24.857 ms 25.695 ms]
- `update_content_entity`   time:   [25.076 ms 25.819 ms 26.584 ms]
- `delete_entity`           time:   [41.999 ms 42.719 ms 43.492 ms] - Filtered 400s
- `evict_entity_id`         time:   [41.387 ms 42.029 ms 42.731 ms] - Filtered 400s
- `evict_entity`            time:   [31.582 ms 31.805 ms 32.039 ms] - Filtered 400s
- `select_all` 20 entities  time:   [28.217 ms 30.311 ms 32.479 ms]
- `select_all` 10 entities  time:   [25.265 ms 26.094 ms 26.932 ms]
- `select_all` 1 entity     time:   [22.129 ms 22.776 ms 23.434 ms]
