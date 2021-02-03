# WooriDB

WooriDB is an immutable time serial database.

## Installation

- `make release` at `./woori-db`; or
- `cargo run --release`
- `cargo run --release` at root or at `./woori-db`;

## Usage



## Transactions:

### Parser
- [ ] [DOING] Woori Query language parser

### Transactions by Query
- [x] Create entity: it is similar to `CREATE TABLE` in SQL. It requires a rntity name like `my_entity_name` after `CREATE ENTITY`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'CREATE ENTITY my_entity_name'`. 
  - [x] Create entity with Unique identifier. This prevents duplciated unique key values, for example if you insert an entity with key `id` containing `123usize` for entity `my_entity` there can be only one entity `id` with value `123` in `my_entity`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'CREATE ENTITY my_entity_name UNIQUES name, ssn'`
- [x] Insert entity: it inserts a HashMap into the entity created (`my_entity_name`). This request returns a `Uuid`. Ecample request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'insert {a: 123,} INTO my_entity_name'`.
- [x] Update entity: There are 2 updates possible.
  - [x] SET: `SET` updates defines the current value of the entity to the ones being passed, so if your entity is `{a: 123, b: 12.5,}` and your set update has the hashmap `{a: 432, c: \"hello\",}`, the current state value will be `{a: 432, b: 12.5, c: \"hello\",}`. Example request:  `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'UPDATE my_entity_name SET {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.
  - [x] CONTENT: `CONTENT` updates are a way to add numerical values and concatenate Strings, so if your entity is `{a: 432, c: \"hello\",}` and your content update has the hashmap `{a: -5, c: \"world\", b: 12.5}` the current state will be `{a: 427, c: \"helloworld\", b: 12.5}`. `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'UPDATE my_entity_name CONTENT {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.
- [x] Match Update: Updates only if precondition is matched, this transaction is significantly slower than others. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'MATCH ALL(a > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. Possible preconditions:
  - `ALL` or `ANY` are required to set preconditions. `ALL` means that a logical `AND`/`&&` will be applied to all conditions and `ANY` means that a logical `OR`/`||` will be applied to all conditions. They contain a series of preconditions separated by `,`. For example `ALL(a > 100, b <= 20.0)` or `ANY(a == "hello", b != true)`.
  - **NULL KEYS**, `ALL` returns error if a null key is present and `ANY` just ignores null keys.
  - `==` means equals, so if `a == 100`, this means that the entity key `a` must equal to `100`.
  - `!=` means not equals, so if `a != 100`, this means that the entity key `a` must not equal to `100`.
  - `>=` means greater or equal, so if `a >= 100`, this means that the entity key `a` must greater or equal to `100`.
  - `<=` means lesser or equal, so if `a <= 100`, this means that the entity key `a` must lesser or equal to `100`.
  - `>` means greater, so if `a > 100`, this means that the entity key `a` must greater than `100`. 
  - `<` means lesser, so if `a < 100`, this means that the entity key `a` must lesser than `100`. 
- [x] Delete entity: This is pretty simple, it deletes the last state of an entity. So if you have one update on you entity it will roll back to the `INSERT` event. However, if you have only an `INSERT` event you state will become an empty hashmap. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'delete 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`
- [ ] Evict entity: Removes all ocurrences of an entity. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'EVICT 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. For now it only deletes the acess to the entity history.
- [ ] Select entities

### SELECT = Functions that could be implemented from Relation Algebra:
- [ ] Select
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

## Benchmark

* `create_entity`
```
time:  [15.443 ms 15.496 ms 15.547 ms]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
```

* `insert_entity`
```
time:   [15.623 ms 15.661 ms 15.699 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
```

### Extra TODOS
- [ ] Test Actors
- [ ] Docs
- [ ] Clippy
- [ ] Transactions endpoints??
<!-- ### Transactions by Endpoint
- [ ] Create entity
- [ ] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities ???? -->
- [ ] Benchmarks