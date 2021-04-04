# WooriDB
[USER GUIDE](https://naomijub.github.io/wooridb/)

WooriDB is a general purpose (**EXPERIMENTAL**) time serial database, which means it contains all entities registries indexed by DateTime. It is schemaless, key-value storage and uses its own query syntax that is similar to SparQL and Crux's Datalog. 

Some other features are:
- Hashing keys content with [`ENCRYPT`](https://github.com/naomijub/wooridb#create-entity) keyword.
- Hashed values are filtered out and can only be checked with  [`CHECK`](https://github.com/naomijub/wooridb#checks-validity-of-of-an-encrypted-key) keyword.
- [`Ron`](https://github.com/ron-rs/ron/blob/master/docs/grammar.md) schemas for input and output.
  - [x] JSON is supported via feature.
  - [ ] EDN to be supported via feature.
- Entities are indexed by `entity_name` (Entity Tree), `DateTime` (Time Serial) and `Uuid` (Entity ID). Entity format is a HashMap where keys are strings and values are supported [`Types`](https://github.com/naomijub/wooridb/blob/main/wql/src/lib.rs#L78).
- Stores persistent data locally.
- Able to handle very large numbers when using the `P` suffix.
  - Ex: `98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P`.
- Configuration is done via environment variables.
- Authentication and Authorization via session token
- [Conditional Update](https://naomijub.github.io/wooridb/sec-6-tx.html#match-update)
- Some Relation Algebra
- Entity history

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
you can find the latest docker image at **[naomijub/wooridb](https://hub.docker.com/repository/docker/naomijubs/wooridb)**. The current most stable tag is **`beta-6`**. To execute the docker container run:

* `docker run -p 1438:1438 naomijubs/wooridb:beta-6 debug` for debug mode.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-6 run`  for size optimization.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-6 release` for performance optimization.
* All `-e/--env` can be replaced by a `--env-file path/to/your/.env`. Your `.env`file should contain the following fields:
```
HASHING_COST=16
PORT=1438
AUTH_HASHING_COST=8
ADMIN=your-admin-id
ADMIN_PASSWORD=your-admin-pswd
``` 

## Usage
* Responses are in [`RON`](https://github.com/ron-rs/ron) format. Support for `JSON` is via `--feature json` and `EDN` will be done later by using features.
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

## Milestone to stable-ish version
- [ ] [issues](https://github.com/naomijub/wooridb/issues?q=is%3Aissue+is%3Aopen+label%3AMilestone)

## Current Benchmarks
> 2,3 GHz Intel Core i9 8-Core 32 GB 2667 MHz DDR4

- `create_entity`                      time:   [4.7008 ms 4.7367 ms 4.7725 ms]
- `insert_entity`                      time:   [9.5938 ms 9.8263 ms 10.057 ms]
- `update_set_entity`                  time:   [11.899 ms 11.984 ms 12.083 ms]
- `update_content_entity`              time:   [12.048 ms 12.328 ms 12.719 ms]
- `delete_entity`                      time:   [5.3966 ms 5.4423 ms 5.4908 ms] - Filtered 400s
- `evict_entity_id`                    time:   [15.534 ms 15.623 ms 15.721 ms] - Filtered 400s
- `evict_entity`                       time:   [12.318 ms 12.416 ms 12.540 ms] - Filtered 400s
- `select_all` 20 entities             time:   [5.8810 ms 5.9318 ms 5.9889 ms]
- `select_all` 10 entities             time:   [5.4012 ms 5.4432 ms 5.4860 ms]
- `select_all` 1 entity                time:   [4.8442 ms 4.8786 ms 4.9140 ms]
- `history_10_registries_for_entity`   time:   [5.7265 ms 5.7868 ms 5.8673 ms]
- `history_20_registries_for_entity`   time:   [6.1166 ms 6.1943 ms 6.2987 ms]

### WQL 
> 2,3 GHz Intel Core i9 8-Core 32 GB 2667 MHz DDR4

- `create_entity`           time:   [433.57 ns 435.00 ns 436.38 ns]
- `inser_entity`            time:   [1.6349 us 1.6406 us 1.6463 us]
- `select_all`              time:   [429.79 ns 431.05 ns 432.14 ns]
- `select_args`             time:   [655.40 ns 657.53 ns 659.71 ns]


### artillery.io 
> 2,3 GHz Intel Core i9 8-Core 32 GB 2667 MHz DDR4

[**Insert**](./insert-report.json)

Config file:
```yml
config:
  target: "http://localhost:1438"
  phases:
    - duration: 100
      arrivalRate: 10
  defaults:
    headers:
      Content-Type: "application/wql"
scenarios:
  - flow:
      - post:
          url: "/wql/tx"
          body: "INSERT {name: \"name\", last_name: \"last name\", age: 20, blood: 'O'} INTO person"
```

[**Select**](./select-report.json)

Contains 1000 registries of `{name: \"name\", last_name: \"last name\", age: 20, blood: 'O'}`.

Config file:
```yml
config:
  target: "http://localhost:1438"
  phases:
    - duration: 100
      arrivalRate: 10
  defaults:
    headers:
      Content-Type: "application/wql"
scenarios:
  - flow:
      - post:
          url: "/wql/query"
          body: "SELECT * FROM person"
```            
