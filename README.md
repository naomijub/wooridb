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


## Installation
- if you don't have Rust and Cargo installed, run `make setup` at root.
- `make run` at root for `release mode`, or `make debug` for `debug mode` (Debug mode doesn't have auth system enabled).

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


## Authentication and Authorization (SIMPLE implementation)
Authentication and authorization only work with release mode, so `cargo run --release` is required. 

Some environment variables are also required:
```
AUTH_HASHING_COST=8
ADMIN=your_admin
ADMIN_PASSWORD=your_password
```

### Creating new users
* `ADMIN` is the only user role capable of creating new users. 

To create a new user, POST at `/auth/createUser` with your admin credentials and the new user info as follows (in RON format):
```ron
(
  admin_id: "your_admin",
  admin_password: "your_password",
  user_info: (
    user_password: "my_password",
    role: [User,],
  ),
)
```
User information consists of the user's password and the user's roles. Remember to always put `,` at the end. 
Response to this request will be `(user_id: \"<some-uuid>\",)`, containing the user's unique ID.

* [ ] Adding other admins and removing admins is not yet implemented.

### Getting a session token
To make a request at WQL endpoints you need a session token that will expire within 3600 seconds. To retrieve a session token you need to PUT at endpoint `/auth/putUserSession` your user credentials as follows (in RON format):
```ron
(id: "<user_id>", user_password: "<user_password>",)
```
Response will be a plain/text with your token.
* [ ] Configure session token expiration time.

### Making auth requests to `/wql/tx` and `/wql/query`.

To avoid authentication and authorization errors, add your token to the authorization bearer header, `Authorization: Bearer <your session token>`. 
Your user needs the correct session token and the correct role for this request.


## Parser

WooriDB uses the [Woori Query language parser](https://github.com/naomijub/wooridb/tree/main/wql), evolving as required.


## Transactions:
> **Reminder**
> A comma is required at the end of every data structure representation.
> Ex.: `{a: 123, b: 456,}`, `#{a, b, c,}`, `(a, b, c,)`. 
> No need for `;` at the end of each expression.

* Endpoint for `CREATE, INSERT, UPDATE, MATCH, DELETE, EVICT`: `<ip>:1438/wql/tx`
* Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/tx -d '...'`


### `CREATE ENTITY`:

Similar to `CREATE TABLE` in SQL, it creates an entity tree. It requires an entity name like `my_entity_name` after `CREATE ENTITY`. Example request: `'CREATE ENTITY my_entity_name'`. 

  * **CREATE ENTITY with UNIQUE IDENTIFIERS**: This prevents duplicated unique entity map keys, for example if you insert an entity with key `id` containing `123usize` for entity `my_entity` there can be only one entity `id` with value `123` in `my_entity`. 
    * Example request: `'CREATE ENTITY my_entity_name UNIQUES #{name, ssn,}'`.
  * **CREATE ENTITY with ENCRYPTED KEYS**: Encrypts entities map values for defined keys. 
    * Example request: `'CREATE ENTITY my_entity_name ENCRYPT #{password, ssn,}'`
  * It is possible to create entities trees with uniques and encryption. `CREATE ENTITY my_entity_name ENCRYPT #{password,} UNIQUES #{name, ssn,}`
    * Example request: `CREATE ENTITY my_entity_name ENCRYPT #{password,} UNIQUES #{name, ssn,}`
  * When the system has encrypted keys, the requests take longer due to hashing function and the verify function. This is determined by the hashing cost:
  ```
  bench_cost_10      ... bench:  51,474,665 ns/iter (+/- 16,006,581)
  bench_cost_14      ... bench: 839,109,086 ns/iter (+/- 274,507,463)
  bench_cost_4       ... bench:     795,814 ns/iter (+/- 42,838)
  bench_cost_default ... bench: 195,344,338 ns/iter (+/- 8,329,675)
  * Note that I don't go above 14 as it takes too long. However, it is way safer, it is a trade-off. 
  ```


### `INSERT ENTITY`:
Inserts a **HashMap<String, [Types](https://github.com/naomijub/wooridb/blob/main/wql/src/lib.rs#L78)>**  into the entity tree created (`my_entity_name`). This request returns a `Uuid` containing the entity id. 

Example request: 
```'insert {a: 123,  c: \"hello\", d: \"world\",} INTO my_entity_name'``` 
The request above will insert an entity as follows:

```
{"my_entity_name": {
  48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, c: \"hello\", d: \"world\",},
}}
```


### `UPDATE ENTITY`: 
There are two possible updates.

#### `UPDATE SET ENTITY`:
`SET` updates defines the current value of the entity map to the ones being passed, so if your entity map is `{a: 123, b: 12.5,}` and your set update has the hashmap `{a: 432, c: \"hello\",}`, the current state of the entity map will be `{a: 432, b: 12.5, c: \"hello\",}`. Example request:  `'UPDATE my_entity_name SET {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.
  
#### `UPDATE CONTENT ENTITY`:
`CONTENT` updates are a way to add numerical values and concatenate Strings, so if your entity map is `{a: 432, c: \"hello\",}` and your content update has the hashmap `{a: -5, c: \"world\", b: 12.5}` the current state of the entity map will be `{a: 427, c: \"helloworld\", b: 12.5}`. `'UPDATE my_entity_name CONTENT {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed'`.


### `MATCH UPDATE ENTITY`:
Updates only if precondition is matched. This transaction is significantly slower than others. 
Example request: `'MATCH ALL(a > 100, b <= 20.0) UPDATE test_match_all SET {{a: 43, c: Nil,}} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. 

Possible preconditions:
  - `ALL` or `ANY` are required to set preconditions. `ALL` means that a logical `AND`/`&&` will be applied to all conditions and `ANY` means that a logical `OR`/`||` will be applied to all conditions. They contain a series of preconditions separated by `,`. For example `ALL(a > 100, b <= 20.0)` or `ANY(a == "hello", b != true)`.
  - **NULL KEYS**, `ALL` returns an error if a null key is present and `ANY` just ignores null keys.
  - `==` means equals, so if `a == 100`, this means that the entity key `a` must equals to `100`.
  - `!=` means not equals, so if `a != 100`, this means that the entity key `a` must not equals to `100`.
  - `>=` means greater or equal, so if `a >= 100`, this means that the entity key `a` must br greater or equals to `100`.
  - `<=` means lesser or equal, so if `a <= 100`, this means that the entity key `a` must be lesser or equals to `100`.
  - `>` means greater, so if `a > 100`, this means that the entity key `a` must be greater than `100`. 
  - `<` means lesser, so if `a < 100`, this means that the entity key `a` must be lesser than `100`.
  - [ ] Possibly be changed to `WHERE` syntax.


### `DELETE ENTITY with ID`:
Deletes the last entity event for an ID, that is, it deletes the last state of an entity map. The deleted state is preserved in the database but cannot be queried anymore.

If you have, for example, one update on your entity, it will roll back to the `INSERT` event. 
However, if you have only an `INSERT` event then your state will become an empty hashmap. 

Example request: `'delete 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`
  - [ ] Delete entity with ID at transaction-time


### `EVICT ENTITY`:

#### `EVICT ENTITY ID`:
Removes all occurrences of an entity map with the given ID. 

Example request `'EVICT 48c7640e-9287-468a-a07c-2fb00da5eaed from my_entity_name'`. 

> For now it only deletes the access to the entity history.

#### `EVICT ENTITY`:
Evicts all registries from entity and removes entity, which means entity tree does not contain the key for the evicted entity: Similar to SQL `DROP TABLE <entity>`. 

Example request: `EVICT my_entity`.



## QUERY:
> Endpoint for `SELECT, CHECK`: `<ip>:1438/wql/query`
> 
> Example request: `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d`

The basic read operation. Its endpoint is `/wql/query`. 

To better understand the next sub-items, lets say the entity tree `my_entity_name` has the following values:
```rust
{"my_entity_name": {
  48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\", d: \"world\",},
  57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\", d: \"brasil\",},
  54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\", d: \"korea\",},
}}
```

### `SELECT`
This is the way to query entities from WooriDB. Similar to SQL and SparQL `SELECT`.

#### SELECTS all keys FROM ENTITY:
Selects all entities ids and maps from entity tree `my_entity`. The `*` means that all the keys from each entity map will be returned. 
It is equivalent to SQL's `Select * From table`. 

Example request: `'SELECT * from my_entity_name'`. This query will return a `BTreeMap<Uuid, HashMap<String, Types>>`:
```
{
  48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\", d: \"world\",}, 57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\", d: \"brasil\",}, 54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\", d: \"korea\",},
}
```


#### SELECTS a set of keys FROM ENTITY:
Select all entities ids and maps from entity tree `my_entity`. Only the keys `a, b, c,` defined by the set `#{a, b, c,}` will be returned for each entity.

It is equivalent to `SELECT a, b, c FROM table`. 

Example request: `'SELECT #{a, b, c,} from my_entity_name'`. 

This query will return `48c7640e-9287-468a-a07c-2fb00da5eaed: {a: 123, b: 43.3, c: \"hello\",}, 57c7640e-9287-448a-d07c-3db01da5earg: {a: 456, b: 73.3, c: \"hello\",}, 54k6640e-5687-445a-d07c-5hg61da5earg: {a: 789, b: 93.3, c: \"hello\",},` 


#### SELECT one entity map with all keys FROM ENTITY:
- Key `ID` is the entity id's Uuid.

Select one entity map (by its ID) from entity tree `my_entity` with all of its keys. 

It is equivalent to SQL's `Select * From table WHERE id = <uuid>`. 

Example request `'SELECT * from my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`. 

This query will return `{a: 123, b: 43.3, c: \"hello\", d: \"world\",}`.


#### SELECT one entity map with a set of keys FROM ENTITY:
- Key `ID` is the entity id's Uuid.

Select one entity  map (by its ID) from entity tree `my_entity` with a set of keys. 

It is equivalent to SQL's `SELECT a, b, c FROM table WHERE id = <uuid>`. 

Example request: `'SELECT #{a, b, c,} from my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`. 

This query will return `{a: 123, b: 43.3, c: \"hello\",}`


#### SELECT a set of entities IDs and maps FROM ENTITY:
- Key `IN` receives a set of Uuids

Select a few entities maps (by their IDs) from entity tree `my_entity`. This requires knowing their IDs. 

Example request: `'SELECT #{a, b, c,} from my_entity_name IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}'` or `'SELECT * from my_entity_name IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}'`.
  

#### SELECTs last entity map BY ID FROM ENTITY AT DATETIME<UTC>:
- Key `WHEN AT` is the date to search. Time will be discarded. 

Select an entity on a defined past day. The `ID` field can be used before `WHEN` to define a specific entity id. 
Date format should be `"2014-11-28T21:00:09+09:00"` or `"2014-11-28T21:00:09Z"`. 
  
Example request: `'Select * FROM my_entity ID 0a1b16ed-886c-4c99-97c9-0b977778ec13 WHEN AT 2014-11-28T21:00:09+09:00'` 
Example request: `'Select #{name,id,} FROM my_entity WHEN AT 2014-11-28T21:00:09Z'`.
  

#### SELECTs all entities maps BY ID FROM ENTITY between two DATETIME<UTC>:
- Key `WHEN` defines it as a temporal query.
- Key `START` is the `DateTime<Utc>` to start the range query.
- Key `END` is the `DateTime<Utc>` to end the range query.

Select all occurrences of an entity id from entity tree `entity_name` in a time range. The time range must be on the same day as `START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z`. 
  
Example request: `'SELECT * FROM entity_name ID <uuid> WHEN START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z'`. 
  

#### SELECT entities ids and maps FROM ENTITY WHERE conditions
- Key `WHERE` receives all clauses inside a `{...}` block.
Selects entities ids and maps with positive WHERE clauses. 

This is probably the most different part in relation to SQL as it is inspired by SparQL and Crux/Datomic datalog. 

To use `select` with the `where` clause you can use the following expressions:
* `SELECT * FROM my_entity WHERE {<clauses>}` 
* `SELECT #{key_1, key_2, key_3,} FROM my_entity WHERE {<clauses>}` 

All clauses should be ended with/separated by `,` and the available functions are `==, !=, >, <, >=, <=, like, between, in, or`. 

To use the functions you need to attribute a key content to a variable, this is done by `?* my_entity:key_1 ?k1` (entity_tree.key:entity_map.key) and then `?k1` can be used as follows:
* `in`: `(in ?k1 123 34543 7645 435)`, where arguments after `?k1` are turned into a set. 
    * Note: **for now, please don't use `,`**.
* `between`: `(between ?k1 0 435)`, after `?k1` the first argument is the `start` value and the second argument is the `end` value.  If you set more than 2 arguments it will return a `ClauseError`.
* `like`: `(like ?k2 "%naomi%")`, like is comparing `?k2` with the string `"%naomi%"` considering that `%` are wildcards. `"%naomi"` means `end_with("naomi")`, `"naomi%"` means `starts_with("naomi")` and `"%naomi%"` means `contains("naomi")`. In the future this will be replaced by regex.
* `==`, `>=`, `>`, `<`, `<=`, `!=` -> `(>= ?k1 0)` which means *all values that `?k1` is greater than or equal to `0`*.
* `or`: All arguments inside the `or` function call will be evaluated to `true` if any of them is `true`. 
    * Example: 
* Every function will be added as logical `and` unless they are inside an `or` function. 
    * Example: `"Select * From test_or WHERE { ?* test_or:a ?a, ?* test_or:c ?c, (== ?a 123), (or (>= c 4300.0) (< c 6.9) ), }`
    * This means that `?a` needs to be equal to 123 **and** `?c` to be greater or equal to 4300.0 `or` `?c` to be lesser than 6.9.
* Missing features: 
    * An `and` inside an `or` block.
    * [ ] Temporality?

```
Select * 
FROM my_entity 
WHERE {
  ?* my_entity:a ?a, // `,` <-
  ?* my_entity:c ?c, // `,` <-
  (== ?a 123), // `,` <-
  (or
      (>= c 4300.0)  // `,` <-
      (< c 6.9) // `,` <-
  ), // `,` <-
}
```

#### SELECT - Functions that could/will be implemented from Relation Algebra:

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


### CHECKs validity of an encrypted key
Checks for encrypted data validity. It requires an entity tree name after `FROM` and an entity id as Uuid after `ID`. This transaction only works with keys that are encrypted and it serves to verify if the passed values are `true` of `false` against encrypted data. Example request: `'CHECK {pswd: \"my-password\", ssn: 3948453,} FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed'`.

## TODOS
- [ ] Read infos from ztsd files [issue 28](https://github.com/naomijub/wooridb/issues/28)
- [x] Docs [issue 31](https://github.com/naomijub/wooridb/issues/31). PRs [README](https://github.com/naomijub/wooridb/pull/54) [First book chapters](https://github.com/naomijub/wooridb/pull/71)
- [ ] Docker
- [ ] Benchmarks. PRs [61](https://github.com/naomijub/wooridb/pull/61)

## Current Benchmarks
>  MacBook Pro, 2.2 GHz Intel Core i7, 16 GB 2400 MHz DDR4

- `create_entity`           time:   [15.269 ms 15.332 ms 15.396 ms]
- `insert_entity`           time:   [27.438 ms 28.177 ms 28.958 ms]
- `update_set_entity`       time:   [39.814 ms 40.054 ms 40.314 ms]
- `update_content_entity`   time:   [42.359 ms 43.129 ms 43.942 ms]
- `delete_entity`           time:   [41.999 ms 42.719 ms 43.492 ms] - Filtered 400s
- `evict_entity_id`         time:   [41.387 ms 42.029 ms 42.731 ms] - Filtered 400s
- `evict_entity`            time:   [31.582 ms 31.805 ms 32.039 ms] - Filtered 400s
