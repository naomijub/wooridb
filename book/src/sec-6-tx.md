# Transactions

Transaction is the name of all operations that change the database state, like `CREATE, INSERT, UPDATE, MATCH, DELETE, EVICT`. This is done by sending a `POST` request to endpoint `<ip>:1438/wql/tx`. An example request would be `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d 'CREATE ENTITY my_entity'`. In `release mode` it is necessary to use header `Authorization: Bearer <your session token>` for this endpoint.

> **Reminder**
> A comma is required at the end of every data structure representation.
> Ex.: `{a: 123, b: 456,}`, `#{a, b, c,}`, `(a, b, c,)`. 
> No need for `;` at the end of each expression.

## `CREATE ENTITY`
[CREATE WQL Reference](./sec-4-wql.md#create)

Similar to `CREATE TABLE` in SQL, it creates an entity tree which matches the table name. It requires an entity name like `my_entity_name` after `CREATE ENTITY`. 

Example request: 
```sql
CREATE ENTITY my_entity_name
``` 

Example response:
```rust
(
 entity: "my_entity_name",
 message: "Entity `my_entity_name` created",
)
```

* `CREATE ENTITY <entity> ENCRYPT #{keys...}`: When the system has encrypted keys, the `insert` and `upload` requests take longer due to hashing function and the verify function. This is determined by the hashing cost:
```
bench_cost_10      ... bench:  51,474,665 ns/iter (+/- 16,006,581)
bench_cost_14      ... bench: 839,109,086 ns/iter (+/- 274,507,463)
bench_cost_4       ... bench:     795,814 ns/iter (+/- 42,838)
bench_cost_default ... bench: 195,344,338 ns/iter (+/- 8,329,675)
* Note that I don't go above 14 as it takes too long. However, it is way safer, it is a trade-off. 
```

## `INSERT`
[INSERT WQL Reference](./sec-4-wql.md#insert)

Inserts a **HashMap<String, [Types](./sec-4-wql.md#entity-map-value-types)>**  into the entity tree key previously created (`my_entity_name`). This request returns a `Uuid` containing the entity id. 

Example request: 
```sql
INSERT {a: 123,  c: \"hello\", d: \"world\",} 
INTO my_entity_name
``` 

Example response:
```rust
(
 entity: "my_entity_name",
 uuid: "00d025c9-eda8-4190-a33a-29998bd77bd3",
 message: "Entity my_entity_name inserted with Uuid 00d025c9-eda8-4190-a33a-29998bd77bd3",
)
```

The request above will insert an entity with the following structure in `my_entity_name`:

```rust
{"my_entity_name": {
    Uuid(00d025c9-eda8-4190-a33a-29998bd77bd3): {a: 123, c: "hello", d: "world",},
}}
```

## `UPDATE`
Updates the content of an entity map for an entity tree key and an entity id. There are two possible updates:

### `UPDATE SET`
[UPDATE SET WQL Reference](./sec-4-wql.md#update-set)
`SET` updates defines the current value of the entity map to the ones being passed, so if your entity map is `{a: 123, b: 12.5,}` and your set update has the hashmap `{a: 432, c: \"hello\",}`, the current state of the entity map will be `{a: 432, b: 12.5, c: \"hello\",}`. 

Example request:  
```sql
UPDATE my_entity_name 
SET {a: -4, b: 32,} 
INTO 00d025c9-eda8-4190-a33a-29998bd77bd3
```

Example response:
```rust
(
 entity: "my_entity_name",
 uuid: "00d025c9-eda8-4190-a33a-29998bd77bd3",
 state: "{\"b\": Integer(32),\"a\": Integer(-4),}",
 message: "Entity my_entity_name with Uuid 00d025c9-eda8-4190-a33a-29998bd77bd3 updated",
)
```

### `UPDATE CONTENT`
[UPDATE CONTENT WQL Reference](./sec-4-wql.md#update-content)
`CONTENT` updates are a way to add numerical values and concatenate Strings, so if your entity map is `{a: 432, c: \"hello\",}` and your content update has the hashmap `{a: -5, c: \"world\", b: 12.5}` the current state of the entity map will be `{a: 427, c: \"helloworld\", b: 12.5}`. 

Example request:
```sql
UPDATE my_entity_name 
CONTENT {a: -34, b: 7,} 
INTO 00d025c9-eda8-4190-a33a-29998bd77bd3
```

Example response:
```rust
(
 entity: "my_entity_name",
 uuid: "00d025c9-eda8-4190-a33a-29998bd77bd3",
 state: "{\"b\": Integer(39),\"a\": Integer(-38),}",
 message: "Entity my_entity_name with Uuid 00d025c9-eda8-4190-a33a-29998bd77bd3 updated",
)
```

## `MATCH UPDATE`
[MATCH UPDATE WQL Reference](./sec-4-wql.md#match-update)

Updates entity only if precondition condition is matched. This transaction is significantly slower than other updates.

Example request:
```sql
MATCH ALL(a < 0, b >= 3) 
UPDATE my_entity_name 
SET {a: 123, g: NiL,} 
INTO 00d025c9-eda8-4190-a33a-29998bd77bd3
```

Example response:
```rust
(
 entity: "my_entity_name",
 uuid: "00d025c9-eda8-4190-a33a-29998bd77bd3",
 state: "{\"b\": Integer(39),\"a\": Integer(123),\"g\": Nil,}",
 message: "Entity my_entity_name with Uuid 00d025c9-eda8-4190-a33a-29998bd77bd3 updated",
)
```

### Issues:
- [ ] [Last argument does not support ending comma](https://github.com/naomijub/wooridb/issues/74)

## `DELETE`
[DELETE WQL Reference](./sec-4-wql.md#delete)

Deletes the last entity event for an ID, that is, it deletes the last state of an entity map. The deleted state is preserved in the database but cannot be queried anymore.

If you have, for example, one update on your entity, it will roll back to the `INSERT` event. 
However, if you have only an `INSERT` event then your state will become an empty hashmap. 

Example request: 
```sql
DELETE 00d025c9-eda8-4190-a33a-29998bd77bd3 FROM my_entity_name
```
  
Example response:
```rust
(
 entity: "my_entity",
 uuid: Some("00d025c9-eda8-4190-a33a-29998bd77bd3"),
 message: "Entity my_entity with Uuid 00d025c9-eda8-4190-a33a-29998bd77bd3 deleted",
)
```

### TODOs:
- [ ] Delete entity with ID at transaction-time

## `EVICT`
[EVICT WQL Reference](./sec-4-wql.md#evict)

### `EVICT ENTITY ID`:
Removes all occurrences of an entity map with the given ID. 

Example request 
```sql
EVICT 00d025c9-eda8-4190-a33a-29998bd77bd3 from my_entity_name
``` 

Example response:
```rust
(
 entity: "my_entity",
 uuid: Some("00d025c9-eda8-4190-a33a-29998bd77bd3"),
 message: "Entity my_entity with id 6ac9d1bb-2b0c-4631-bc05-682ab4ae8306 evicted",
)
```

> For now it only deletes the access to the entity history.

### `EVICT ENTITY`:
Evicts all entity ids registries from entity tree and removes entity tree key, which means entity tree does not contain the key for the evicted entity: Similar to SQL `DROP TABLE <entity>`. 

Example request: 
```sql
EVICT my_entity
```

Example response:
```rust
(
 entity: "my_entity",
 uuid: None,
 message: "Entity my_entity evicted",
)
```