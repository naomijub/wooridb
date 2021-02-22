# Woori Query Language

[Woori Query language](https://github.com/naomijub/wooridb/tree/main/wql) or `WQL` is WooriDB's Query Language and it is inspired by SparQL, Datalog and SQL. Its main features are:

**Transactions**
- `CREATE` entity tree key by name.
    - `UNIQUE`: With unique values for entity map keys inside entity tree.
    - `ENCRYPTS`: With encrypted values for defined key-values inside entity map.
- `INSERT` entity map into entity tree.
- `UPDATE`s with `SET` or `CONTENT` entity map.
    - SET UPDATE replaces the sent entity map as the entity's map content.
    - CONTENT UPDATE updates numerical and string the current entity's map content with the sent entity map value and the other values work the same way as SET. 
- `MATCH UPDATE` updates entity map content with new content if match condition is satisfied.
- `DELETE`s the last entity map content for and entity id.
- `EVICT`
    - Evicts specific entity id and entity map
    - Evicts all entities in the entity tree key.

**Queries**
- `SELECT` the only way to retrieve entity's content.
- `CHECK` the only way to verify keys that are encrypted.

> ALL DATA STRUCTURES HASHMAPS, HASHSETS AND LIST MUST CONTAIN A `,` AFTER EACH ELEMENT. Example `#{name, ssn,}` is valid but `#{name, ssn}` is not valid.

## Examples

### CREATE
Creates an entity tree key.

* `CREATE ENTITY my_entity` this will create an entity tree key named `my_entity`, in SQL terms it means `CREATE TABLE my_entity`.
* `CREATE ENTITY my_entity UNIQUES #{name, ssn,}` the entity tree key named `my_entity` will only allow unique values for the entities keys `name` and `ssn` in its maps.
* `CREATE ENTITY my_entity ENCRYPTS #{pswd,}` the entity tree key named `my_entity` will encrypt the entities keys that are `pswd`. The hashing cost of the encrypt is defined by the environment variable `HASHING_COST`, recommended is between 10 and 14.
* Encryted keys cannot be uniques so `CREATE ENTITY my_entity UNIQUES #{name, ssn, pswd,} ENCRYPTS #{pswd,}` is invalid but `CREATE ENTITY my_entity UNIQUES #{name, ssn,} ENCRYPTS #{pswd,}` is valid.

### INSERT
Inserts an entity id and an entity map into entity tree key.

* `INSERT {a: 123, b: "hello julia",} INTO entity_key` this will insert the entity map `{a: 123, b: "hello julia",}` (key `a` containing as `Type::Integer(123)` and key `b` containing a `Type::String("hello julia")`) and a random Uuid for entity ID into entity tree key `entity_key`.

### UPDATE SET
Updates the content by replacing the previous entity map in entity tree key `my_entity_name` with the entity id `48c7640e-9287-468a-a07c-2fb00da5eaed`.

* `UPDATE my_entity_name SET {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed` this will replace the current entity map stored in entity id `48c7640e-9287-468a-a07c-2fb00da5eaed`.

### UPDATE CONTENT
Updates the content by numerical addition or string concatenation of the previous entity map in entity tree key `my_entity_name` with the entity id `48c7640e-9287-468a-a07c-2fb00da5eaed`. Non numerical or non string value will just be replaced. If key doesn't  exist it will be created.

* `UPDATE my_entity_name CONTENT {a: -4, b: 32,} INTO 48c7640e-9287-468a-a07c-2fb00da5eaed` this will add `-4` to entity map key `a` and add `32` to entity map key `b` in the current entity map stored in entity id `48c7640e-9287-468a-a07c-2fb00da5eaed`.

### MATCH UPDATE
Similar to SET, but it requires a pre-condition to be satisfied.

* `MATCH ALL(a == 1, b >= 3, c != \"hello\", d < 7,) UPDATE this_entity SET {a: 123, g: NiL,} INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536` if all conditions defined inside `ALL` are satisfied the set update will happen.
    - `ALL` is an logical `and` between all conditions, meaning that all of them must be true.
    - `ANY` is an logical `or` between all conditions, meaning that at least one of them must be true.
    - **NULL KEYS**, `ALL` returns an error if a null key is present and `ANY` just ignores null keys.
    - Possible conditions are:
        - `==` means equals, so if `a == 100`, this means that the entity map key `a` must equals to `100`.
        - `!=` means not equals, so if `a != 100`, this means that the entity map key `a` must not equals to `100`.
        - `>=` means greater or equal, so if `a >= 100`, this means that the entity map key `a` must br greater or equals to `100`.
        - `<=` means lesser or equal, so if `a <= 100`, this means that the entity map key `a` must be lesser or equals to `100`.
        - `>` means greater, so if `a > 100`, this means that the entity map key `a` must be greater than `100`. 
        - `<` means lesser, so if `a < 100`, this means that the entity map key `a` must be lesser than `100`.

### DELETE
Deletes the last entity map event for an entity ID in entity tree key, that is, it deletes the last state of an entity map.

* `DELETE 48c7640e-9287-468a-a07c-2fb00da5eaed FROM my_entity_name` this will delete the last state of entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` in entity tree key `my_entity_name` from entity history.

### EVICT
Removes all occurrences of an entity from the entity tree. It can be just the entity id or the whole entity tree key.

* `EVICT 48c7640e-9287-468a-a07c-2fb00da5eaed FROM my_entity_name` removes all occurrences of the entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` from the entity tree key `my_entity_name`, they cannot be queried anymore.
* `EVICT my_entity` removes the key `my_entity` from the entity tree. It cannot be queried anymore. It is similar to SQL's `DROP TABLE my_entity`.

### CHECK
Checks for encrypted key data validity. This transaction only works with keys that are encrypted and it serves  as a way to verify if the passed values are `true` of `false` against encrypted data. 

* `CHECK {pswd: "my-password", ssn: 3948453,} FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed` this will check if keys `psdw` and `ssn` from entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` in entity tree key `my_entity_name` have the values `"my-password"` for pswd and `3948453` for ssn.

### SELECT
This is the way to query entities from WooriDB. Similar to SQL and SparQL `SELECT`.

Possible `SELECT`  combinantions:
* `SELECT * FROM my_entity_name` selects all entity ids and entity maps for the entity tree key `my_entity_name` with all the possible entities map keys.
* `SELECT #{name, last_name, age,} FROM my_entity_name` selects all entity ids and entity maps for the entity tree key `my_entity_name` with only the keys `name, last_name, age` for the entities map.
* `SELECT * FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed` selects the entity map containing the entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` from the entity tree key `my_entity_name` with all the possible entities map keys.
* `SELECT #{name, last_name, age,} FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed` selects the entity map containing the entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` from the entity tree key `my_entity_name` with only the keys `name, last_name, age` for the entities map.
* `SELECT * FROM my_entity_name IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}` this will return the entities map containing the entities ids `#{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg, 54k6640e-5687-445a-d07c-5hg61da5earg,}` from entity tree key `my_entity_name`. Keys set is available.
* `Select * FROM my_entity ID 0a1b16ed-886c-4c99-97c9-0b977778ec13 WHEN AT 2014-11-28T21:00:09+09:00` this will select the last entity map state for the entity id `0a1b16ed-886c-4c99-97c9-0b977778ec13` in entity tree key `my_entity` at date `2014-11-28`. Requires to use DateTime UTC, for now.
* `SELECT * FROM entity_name ID <uuid> WHEN START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z` this will select the all entity map states for the entity id `0a1b16ed-886c-4c99-97c9-0b977778ec13` in entity tree key `my_entity` in the time range starting at `2014-11-28T09:00:09Z` and ending at `2014-11-28T21:00:09Z`.
* `SELECT * FROM my_entity WHERE { ?* my_entity:a ?a, ?* my_entity:c ?c, (== ?a 123),(or (>= ?c 4300.0), (< ?c 6.9),),}` this will select all enitities ids and entities maps from entity tree key `my_entity` that satisfy the where clause.
     - `?* my_entity:a ?a` and `?* my_entity:c ?c` define that the entity keys `a` and `c` from entity tree key `my_entity` will receive the attributed value `?a` and `?c` repectively.
     - `(== ?a 123)` selects all entities which entity map key `a` is equal to `123`.
     - `(or (>= ?c 4300.0), (< ?c 6.9),)` selects all entities which entity map key `c` is greater or equal to `4300.0` **or** is smaller than `6.9`.

#### WHERE Clause
Possible functions for the where clause:
* `in`: `(in ?k1 123 34543 7645 435)`, `?k1` must be present in the set containing `123 34543 7645 435`. NOTE: **for now, please don't use `,`**.
* `between`: `(between ?k1 0 435)`, `?k1`  must be between starting value `0` and ending value `435`. If you set more than 2 arguments it will return a `ClauseError`.
* `like`: `(like ?k2 "%naomi%")`, like is comparing `?k2` with the string `"%naomi%"` considering that `%` are wildcards. `"%naomi"` means `end_with("naomi")`, `"naomi%"` means `starts_with("naomi")` and `"%naomi%"` means `contains("naomi")`. Possible regex support in the future.
* `==`, `>=`, `>`, `<`, `<=`, `!=` -> `(>= ?k1 0)` which means *get all values that `?k1` is greater than or equal to `0`*.
* `or`: All arguments inside the `or` function call will be evaluated to `true` if any of them is `true`. 

#### Relation Algebra
Some relation algebra may be implemented:
- [ ] Projection
- [ ] Union
- [ ] Intersection
- [ ] Difference (SQL's EXCEPT?)
- [ ] Join
- [ ] Product (SQL's CROSS JOIN?)
- [ ] Assign
- [ ] Dedup
- [ ] Sort
- [ ] Aggregate
- [ ] Division

### Entity map value TYPES
> **Types Notes**
> 1. **Precise floats** or **numbers larger than f64::MAX/i128::MAX** can be defined with an UPPERCASE `P` at the end. 
>       * _Note_: This type cannot be updated with `UPDATE CONTENT`. 
>       * Ex.: `INSERT {a: 98347883122138743294728345738925783257325789353593473247832493483478935673.9347324783249348347893567393473247832493483478935673P, } INTO my_entity`.
> 
> 2. `BLOB` will not be supported. Check out [To BLOB or Not To BLOB: Large Object Storage in a Database or a Filesystem](https://www.microsoft.com/en-us/research/publication/to-blob-or-not-to-blob-large-object-storage-in-a-database-or-a-filesystem/)

- [x] `Char(char)` contains the type char defined by `'c'`,
- [x] `Integer(isize)` contains the type isize, just a number without `.`,
- [x] `String(String)` contains any value passed wuth `"this is a string"`,
- [x] `Uuid(Uuid)` contains an `Uuid V4`,
- [x] `Float(f64)` contains the type f64, any number containing `.`,
- [x] `Boolean(bool)` contains type boolean `true` of `false`,
- [x] `Vector(Vec<Types>)` contains a vector of `Types`,
- [x] `Map(HashMap<String, Types>)` contains a HashMap of key `String` and value `Types`,
- [x] `Hash(String)` contains a Hash generated by `ENCRYPTS`,
- [x] `Precise(String)` contains a very large integer or a very large float,
- [x] `Nil` contains a `null/nil` value,
- [ ] `DateTime` to be added.