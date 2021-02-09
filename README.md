# WooriDB

WooriDB is an (EXPERIMENTAL) immutable time serial database. This project is hugely inspired by:
- [Crux](https://github.com/juxt/crux); 
- [Datomic](https://www.datomic.com/); 
- [Prometheus](https://github.com/prometheus/prometheus) 
- [SparkQL](https://en.wikipedia.org/wiki/SPARQL).
- [Database Internals](https://www.amazon.com.br/Database-Internals-Alex-Petrov/dp/1492040347/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Database+Internals%3A&qid=1612831621&sr=8-1)
- [Database System Concept](https://www.amazon.com.br/dp/B073MPV4YC/ref=dp-kindle-redirect?_encoding=UTF8&btkr=1)
- [Designing Data Intensive Application](https://www.amazon.com.br/Designing-Data-Intensive-Applications-Reliable-Maintainable-ebook/dp/B06XPJML5D/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Designing+Data%E2%80%93Intensive+Applications&qid=1612831724&s=books&sr=1-1)
- Professor Andy Pavlo Database classes. 

`Woori` mean `our` and although I developed this DB initially alone, it is in my culture to call everything that is done for our community and by our community **ours**.

## Installation
- if you don't have Rust and Cargo installed run `make setup` at root.
- `make run` at root.

## Usage
* Responses are in `Ron` format, support for `JSON` and `EDN` will be done later by using features.
* For now only persistent local memory is used. Support for `S3`, `Postgres` and `DynamoDB` will be done later by using features.
* Precise floats or number larger than f64::MAX/i128::MAX can be defined with an UPPERCASE `P` at the end. This type cannot be updated with `UPDATE CONTENT`. Example `INSERT {a: 98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P, } INTO my_entity`.
* `BLOB` will not be supported. Checkout *To BLOB or Not To BLOB: Large Object Storage in a Database or a Filesystem*, Russel Sears, Catherine van Ingen, Jim Gray, MSR-TR-2006-45.
* More info at **TODOS**.

## Transactions:
> **Reminder**
> At the end of every data structure representation a `,` (comma) is required. `{a: 123, b: 456,}`, `#{a, b, c,}`, `(a, b, c,)`. No need for `;` at the end of each expression.

### Parser
- [x] Woori Query language parser

### Transactions by type
- [x] Create entity: it is similar to `CREATE TABLE` in SQL. It requires an entity name like `my_entity_name` after `CREATE ENTITY`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name'`. 
  - [x] Create entity with Unique identifier. This prevents duplciated unique key values, for example if you insert an entity with key `id` containing `123usize` for entity `my_entity` there can be only one entity `id` with value `123` in `my_entity`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name UNIQUES #{name, ssn,}'`
  - [x] Encrypt entities keys. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name ENCRYPT #{password, ssn,}'`
  - It is possible to create entities with uniques and encryption. `CREATE ENTITY my_entity_name ENCRYPT #{password,} UNIQUES #{name, ssn,}`
  - When the system has encrypted keys, the requests take longer due to hashing function and the verify function. This is determined by the hashing cost:
  ```
  bench_cost_10      ... bench:  51,474,665 ns/iter (+/- 16,006,581)
  bench_cost_14      ... bench: 839,109,086 ns/iter (+/- 274,507,463)
  bench_cost_4       ... bench:     795,814 ns/iter (+/- 42,838)
  bench_cost_default ... bench: 195,344,338 ns/iter (+/- 8,329,675)
  * Note that I don't go above 14 as it takes too long. But is way safer, it is a trade-off. 
  ```

- [x] Insert entity: it inserts a HashMap into the entity created (`my_entity_name`). This request returns a `Uuid`. Ecample request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'insert {a: 123,  c: \"hello\", d: \"world\",} INTO my_entity_name'`. This will insert an entity as follows:
```
{"my_entity_name": {
  48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, c: \"hello\", d: \"world\",},
}}
```

- [x] Update entity: There are 2 updates possible.

  - [x] SET: `SET` updates defines the current value of the entity to the ones being passed, so if your entity is `{a: 123, b: 12.5,}` and your set update has the hashmap `{a: 432, c: \"hello\",}`, the current state value will be `{a: 432, b: 12.5, c: \"hello\",}`. Example request:  `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'UPDATE my_entity_name SET {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.
  - [x] CONTENT: `CONTENT` updates are a way to add numerical values and concatenate Strings, so if your entity is `{a: 432, c: \"hello\",}` and your content update has the hashmap `{a: -5, c: \"world\", b: 12.5}` the current state will be `{a: 427, c: \"helloworld\", b: 12.5}`. `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'UPDATE my_entity_name CONTENT {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.

- [x] Match Update: Updates only if precondition is matched, this transaction is significantly slower than others. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'MATCH ALL(a > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. Possible preconditions:
  - `ALL` or `ANY` are required to set preconditions. `ALL` means that a logical `AND`/`&&` will be applied to all conditions and `ANY` means that a logical `OR`/`||` will be applied to all conditions. They contain a series of preconditions separated by `,`. For example `ALL(a > 100, b <= 20.0)` or `ANY(a == "hello", b != true)`.
  - **NULL KEYS**, `ALL` returns error if a null key is present and `ANY` just ignores null keys.
  - `==` means equals, so if `a == 100`, this means that the entity key `a` must equal to `100`.
  - `!=` means not equals, so if `a != 100`, this means that the entity key `a` must not equal to `100`.
  - `>=` means greater or equal, so if `a >= 100`, this means that the entity key `a` must greater or equal to `100`.
  - `<=` means lesser or equal, so if `a <= 100`, this means that the entity key `a` must lesser or equal to `100`.
  - `>` means greater, so if `a > 100`, this means that the entity key `a` must greater than `100`. 
  - `<` means lesser, so if `a < 100`, this means that the entity key `a` must lesser than `100`. 

- [x] Delete last entity event: This is pretty simple, it deletes the last state of an entity. So if you have one update on you entity it will roll back to the `INSERT` event. However, if you have only an `INSERT` event you state will become an empty hashmap. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'delete 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`

- [x] Evict entity: Removes all ocurrences of an entity. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'EVICT 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. For now it only deletes the acess to the entity history.
- [x] Evict entity registry: Similar to SQL `DROP TABLE <entity>`.

- [x] Select entities: The basic read operation. Endpoint is `/wql/query`. To better udnerstand the next sub-items, lets say the entity `my_entity_name` has the following values:
```
{"my_entity_name": {
  48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\", d: \"world\",},
  57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\", d: \"brasil\",},
  54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\", d: \"korea\",},
}}
```

  - [x] Select All entities from Entity with all keys for each entity. This operation selects all entities from an entity key. It is equivalent to `Select * From table`. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT * from my_entity_name'`. This query will return `48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\", d: \"world\",}, 57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\", d: \"brasil\",}, 54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\", d: \"korea\",},`.
  - [x] Select All entities from Entity with a set of keys for each entity. This operation selects all entities from an entity key with restricted keys in the output. It is equivalent to `SELECT a, b, c FROM table`.  Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT #{a, b, c,} from my_entity_name'`. This query will return `48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\",}, 57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\",}, 54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\",},` 
  - [x] Select one entity from Entity with all key_values. This operation selects one entity defined by its `ID`. It is equivalent to `Select * From table WHERE id = <uuid>`. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT * from my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`. This query will return `{a: 123, b: 43.3, c: \"hello\", d: \"world\",}`.
  - [x] Select one entity from Entity with a set of key_values. This operation selects one entity defined by its `ID` with restricted keys in the output. It is equivalent to `SELECT a, b, c FROM table WHERE id = <uuid>`.  Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT #{a, b, c,} from my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`. This query will return `{a: 123, b: 43.3, c: \"hello\",}`
  - [x] Select a few entities from entity, knowing their IDs. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT #{a, b, c,} from my_entity_name IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}'` or `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT * from my_entity_name IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}'`.
  - [x] Select an entity at past a day. `ID` field can be used before `WHEN` to define and specific entity. Date format should be `"2014-11-28T21:00:09+09:00"` or `"2014-11-28T21:00:09Z"`. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'Select * FROM my_entity ID 0a1b16ed-886c-4c99-97c9-0b977778ec13 WHEN AT 2014-11-28T21:00:09+09:00'` or something like `'Select #{name,id,} FROM my_entity WHEN AT 2014-11-28T21:00:09Z'`.
  - [x] Select a specific entity in a time range. The time range must be at the same day like `START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT * FROM entity_name ID <uuid> WHEN START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z'`. 
  - [ ] Selects with WHERE?

- [x] Check for encrypted data validity. This transaction only works with keys that are encrypted and it serves to verify if the passed values are `true` of `false`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CHECK {pswd: \"my-password\", ssn: 3948453,} FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`.

- [ ] Multiple queries/tx.
  
  ### SELECT = Functions that could be implemented from Relation Algebra:
- [x] Select
- [ ] Projection
- [ ] Union
- [ ] Intersection
- [ ] Difference (SQL's EXCEPT?)
- [ ] Join
- [ ] Product (SQL's CROSS JOIN?)
- [ ] Rename
- [ ] Assign
- [ ] Dedup
- [ ] Sort
- [ ] Aggregate
- [ ] Division

### TODOS
- [ ] Crash recovery [issue 25](https://github.com/naomijub/wooridb/issues/25)
- [ ] Authentication [issue 26](https://github.com/naomijub/wooridb/issues/26)
- [ ] Read infos from ztsd files [issue 28](https://github.com/naomijub/wooridb/issues/28)
- [ ] Use tokio::sync::Mutex instead of sync (problem is the usage with actors...) [issue 29](https://github.com/naomijub/wooridb/issues/29)
- [ ] Docs [issue 31](https://github.com/naomijub/wooridb/issues/31)
- [ ] Docker
- [ ] Test all Actors
- [ ] Clippy
- [ ] Benchmarks
