# Woori Query Language

Woori Query Language or `WQL` is WooriDB's Query Language and it is inspired by SparQL, Datalog and SQL. Its main features are:

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

* `MATCH ALL(a == 1, b >= 3, c != \"hello\", d < 7,) UPDATE this_entity SET {a: 123, g: NiL,} INTO d6ca73c0-41ff-4975-8a60-fc4a061ce536"` if all conditions defined inside `ALL` are satisfied the set update will happen.
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

