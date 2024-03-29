# Features

- Sintax inspired by SparQL and Crux's datalog.
- Entities are indexed by date(time).
- Schemaless.
- Deep Key-value storage, which means that you can have key values inside hashmap values.
- Hashing entity map keys content with [`ENCRYPT`](https://github.com/naomijub/wooridb#create-entity).
- Unique entity maps keys content with [`UNIQUE`](https://github.com/naomijub/wooridb#create-entity) for an entity tree.
- Hashed values are filtered out of `SELECT` and can only be checked with  [`CHECK`](https://github.com/naomijub/wooridb#checks-validity-of-of-an-encrypted-key) keyword.
- [`Ron`](https://github.com/ron-rs/ron/blob/master/docs/grammar.md) schemas for input and output.
  - [x] JSON is supported via feature.
  - [ ] EDN to be supported via feature.
- Entities are also indexed by `entity_name` (entity tree key) and `Uuid` (entity id). Entity map format is a HashMap where keys are strings and values are supported [`Types`](https://github.com/naomijub/wooridb/blob/main/wql/src/lib.rs#L78).
- Stores persistent data locally.
  - [ ] `S3` as a backend is to be developed.
  - [ ] `DynamoDB` as a backend is to be developed.
- Able to handle very large numbers when using the `P` suffix.
  - Ex: `98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P`.
- Configuration is done via environment variables.
  - [ ] CORS
- Authentication and Authorization via session token
- Users created and deleted by ADMIN user.
- [Conditional Update](https://github.com/naomijub/wooridb#match-update-entity)
- File compression done with `zstd`.
- [x] Entity id history
- [ ] Possible Relation Algebra
    - [x] Union by entity_id
    - [x] Intersection by entity_id
    - [x] Difference by entity_id
    - [ ] Join
    - [x] Dedup
    - [x] Sort
    - [x] Aggregate


## Naming conventions:
- Entity Tree is similar to SQL table, it is the data structure that contains all ids and entities map relations.
- Entity ID is the id of an entity inside Entity tree.
- Entity map is the content of and entity associated with the entity id.
