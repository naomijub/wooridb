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
* `CREATE ENTITY my_entity` this will create an entity tree key named `my_entity`, in SQL terms it means `CREATE TABLE my_entity`.
* `CREATE ENTITY my_entity UNIQUES #{name, ssn,}` the entity tree key named `my_entity` will only allow unique values for the entities keys `name` and `ssn` in its maps.
* `CREATE ENTITY my_entity ENCRYPTS #{pswd,}` the entity tree key named `my_entity` will encrypt the entities keys that are `pswd`. The hashing cost of the encrypt is defined by the environment variable `HASHING_COST`, recommended is between 10 and 14.
* Encryted keys cannot be uniques so `CREATE ENTITY my_entity UNIQUES #{name, ssn, pswd,} ENCRYPTS #{pswd,}` is invalid but `CREATE ENTITY my_entity UNIQUES #{name, ssn,} ENCRYPTS #{pswd,}` is valid.
