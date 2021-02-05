# WooriDB

WooriDB is an immutable time serial database.

## Installation
- if you don't have Rust and Cargo installed run `make setup` at root.
- `make run` at root.

## Usage


## Transactions:

### Parser
- [ ] Woori Query language parser **[DOING]**

### Transactions by type
- [x] Create entity: it is similar to `CREATE TABLE` in SQL. It requires a rntity name like `my_entity_name` after `CREATE ENTITY`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name'`. 
  - [x] Create entity with Unique identifier. This prevents duplciated unique key values, for example if you insert an entity with key `id` containing `123usize` for entity `my_entity` there can be only one entity `id` with value `123` in `my_entity`. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name UNIQUES name, ssn'`
  - [ ] Encrypt entities keys. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity_name ENCRYPT password, ssn'`

- [x] Insert entity: it inserts a HashMap into the entity created (`my_entity_name`). This request returns a `Uuid`. Ecample request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'insert {a: 123,} INTO my_entity_name'`.

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
  <!-- - [ ] Delete entity at specified time. Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'delete 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name AT <DATE-TIME>'` -->

- [x] Evict entity: Removes all ocurrences of an entity. Example request `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'EVICT 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. For now it only deletes the acess to the entity history.
- [x] Evict entity registry: Similar to SQL `DROP TABLE <entity>`.

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

### Extra TODOS
- [ ] Test all Actors
- [ ] Docs
- [ ] Read infos from ztsd files
- [ ] Use tokio::sync::Mutex instead of sync (problem is the usage with actors...)
- [ ] Clippy
- [ ] Benchmarks
- [ ] Transactions endpoints??
<!-- ### Transactions by Endpoint
- [ ] Create entity
- [ ] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities ???? -->