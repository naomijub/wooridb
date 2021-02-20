# Woori Query Language

Woori Query Language or `WQL` is WooriDB's Query Language and it is inspired by SparQL, Datalog and SQL. Its main features are:

**Transactions**
- `CREATE` entity by name.
    - `UNIQUE`: With unique values for a key.
    - `ENCRYPTS`: With encrypted values for defined key-values.
- `INSERT` entity map.
- `UPDATE`s with `SET` or `CONTENT` entity map.
    - SET UPDATE stores the sent entity map as the entity's content.
    - CONTENT UPDATE updates the current entity's content with the sent entity map value. 
- `MATCH UPDATE` updates entity content with new content if match condition is satisfied.
- `DELETE`s the last entity content.
- `EVICT`
    - Evicts specific entity
    - Evicts all entities in the entity tree.

**Queries**
- `SELECT` the only way to retrieve entity's content.
- `CHECK` the only way to verify keys that are encrypted.
    