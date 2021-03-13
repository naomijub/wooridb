# Queries

Query is the name of all operations that read the database, like `SELECT, CHECK`. This is done by sending a `POST` request to endpoint `<ip>:1438/wql/query`. An example request would be `curl -X POST -H "Content-Type: application/wql" <ip>:1438/wql/query -d 'SELECT * FROM my_entity'`. In `release mode` it is necessary to use header `Authorization: Bearer <your session token>` for this endpoint.

> **Reminder**
> A comma is required at the end of every data structure representation.
> Ex.: `{a: 123, b: 456,}`, `#{a, b, c,}`, `(a, b, c,)`. 
> No need for `;` at the end of each expression.

## `CHECK`
[CHECK WQL Reference](./sec-4-wql.md#check)

Checks for encrypted data, in entity map, validity. It requires an entity tree name after `FROM` and an entity id as Uuid after `ID`. This transaction only works with keys that are encrypted and it serves to verify if the passed values are `true` of `false` against encrypted data. 

Considering entity tree key `my_entity_name` with entity id `48c7640e-9287-468a-a07c-2fb00da5eaed` with entity map `{pswd: Hash("my-password"), name: "Julia"}`

### Example 1:
Example request: 
```sql
CHECK {pswd: \"my-password\",} 
FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed
```

Example  response:
```rust
{"pswd": true,}
```

### Exmample 2:
Example request: 
```sql
CHECK {pswd: \"my-password\", ssn: "1234",} 
FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed
```

Example  response:
```rust
(
 error_type: "CheckNonEncryptedKeys",
 error_message: "CHECK can only verify encrypted keys: [\"ssn\"]",
)
```

### Example 3:
Example request: 
```sql
CHECK {pswd: \"your-password\",} 
FROM my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed
```

Example  response:
```rust
{"pswd": false,}
```

## `SELECT`
[SELECT WQL Reference](./sec-4-wql.md#select)

This is the way to query entities from WooriDB. Similar to SQL and SparQL `SELECT`. There are several different keywords that can be combined with `SELECT`, a few examples are:

> `*` and `#{keys...}` can be used in all select modes.

### SELECTing all entity map keys FROM entity tree key:
Same as SQL using the token `*` will defined that all keys in the entity map will be returned. The query `SELECT * FROM entity_name` selects  all entity ids and entity maps found inside entity tree key `entity_name`. It is equivalent to SQL's `Select * From table`. 

Example request: `SELECT * from my_entity_name`. 

Example response:
> This query will return a `BTreeMap<Uuid, HashMap<String, Types>>`
```rust
{
  48c7640e-9287-468a-a07c-2fb00da5eaed: 
    {a: 123, b: 43.3, c: "hello", d: "world",}, 
  57c7640e-9287-448a-d07c-3db01da5earg: 
    {a: 456, b: 73.3, c: "hello", d: "brasil",}, 
  54k6640e-5687-445a-d07c-5hg61da5earg: 
    {a: 789, b: 93.3, c: "hello", d: "korea",},
}
```

### SELECTing a set of entity map keys FROM entity tree key:
Differently from SQL, WQL requires the keys to be inside a set like `#{a, b, c,}`, which will return only the keys `a, b, c`. It is equivalent to `SELECT a, b, c FROM table`. 

Example request: `SELECT #{a, b, c,} FROM my_entity_name`. 

Example response:
```rust
{
    48c7640e-9287-468a-a07c-2fb00da5eaed: 
        {a: 123, b: 43.3, c: "hello",}, 
    57c7640e-9287-448a-d07c-3db01da5earg: 
        {a: 456, b: 73.3, c: "hello",}, 
    54k6640e-5687-445a-d07c-5hg61da5earg: 
        {a: 789, b: 93.3, c: "hello",},
}
``` 

### SELECTing one entity map FROM entity tree key:
Select one entity map (by its ID) from entity tree `my_entity`. By including the key `ID` after the `FROM entity_name` it is possible to select a single entity. The content for `ID`is the entity id's Uuid. It is equivalent to SQL's `Select * From table WHERE id = <uuid>`. 

Example request `SELECT * from my_entity_name ID 48c7640e-9287-468a-a07c-2fb00da5eaed`. 

Example response:
> It will return only the entity map contained inside inside entity id `48c7640e-9287-468a-a07c-2fb00da5eaed`.
`{a: 123, b: 43.3, c: \"hello\", d: \"world\",}`.

### SELECTing a set of entities IDs and maps FROM entity tree key:
Select a few entities maps (by their IDs) from entity tree `my_entity`. Key `IN` receives a set of Uuids

Example request: 
```sql
SELECT #{a, b, c,} 
FROM my_entity_name 
IDS IN #{48c7640e-9287-468a-a07c-2fb00da5eaed, 57c7640e-9287-448a-d07c-3db01da5earg}
```
  
Example response:
```rust
{
    48c7640e-9287-468a-a07c-2fb00da5eaed: 
        {a: 123, b: 43.3, c: "hello",}, 
    57c7640e-9287-448a-d07c-3db01da5earg: 
        {a: 456, b: 73.3, c: "hello",}, 
}
```

### SELECTing the last entity map for entity id at DATETIME<UTC> FROM entity tree key:
Select an entity on a defined past day using the `WHEN AT` keys.  Key `WHEN AT` is the date to search. Time will be discarded. The `ID` field can be used before `WHEN` to define a specific entity id, `IDS IN` is not supported. Date format should be `"2014-11-28T21:00:09+09:00"` or `"2014-11-28T21:00:09Z"`. 
  
Example requests: 
* `Select * FROM my_entity ID 0a1b16ed-886c-4c99-97c9-0b977778ec13 WHEN AT 2014-11-28T21:00:09+09:00` 
* OR `Select #{name,id,} FROM my_entity WHEN AT 2014-11-28T21:00:09Z`.

Example response:
`{a: 34, b: 4.3, c: \"hello\", d: \"Julia\",}`

### TODOs:
- [ ] Support `IDS IN`

### SELECTing all entities maps BY ID FROM ENTITY between two DATETIME<UTC>:
Select all occurrences of an entity id from entity tree `entity_name` in a time range. The time range must be on the same day as `WHEN START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z`. 

- Key `WHEN` defines it as a temporal query.
- Key `START` is the `DateTime<Utc>` to start the range query.
- Key `END` is the `DateTime<Utc>` to end the range query.
- Same day validation occurs. Returning the error message `"START date and END date should be the same date."`.
- `IDS IN` will not be supported as the wuery is too extensive.
  
Example request: 
```sql
SELECT * 
FROM entity_name 
ID 0a1b16ed-886c-4c99-97c9-0b977778ec13 
WHEN START 2014-11-28T09:00:09Z END 2014-11-28T21:00:09Z
``` 

Example response:
```rust
{
    "2014-11-28T09:00:09Z": 
        {a: 34, b: 4.3, c: "hello", d: "Julia",},
    "2014-11-28T13:00:09Z": 
        {a: 23, b: -3.3, c: "hello", d: "World",},
    "2014-11-28T19:00:09Z": 
        {a: 78, b: 67.3, c: "hello", d: "Julia",},
    "2014-11-28T21:00:09Z":
        {a: 123, b: 43.3, c: "hello", d: "Gasp",},
}
```

### SELECTing entities ids and maps FROM entity tree key WHERE conditions are satisfied
This is probably the most different part in relation to SQL as it is inspired by SparQL and Crux/Datomic datalog. Selects entities ids and maps with positive WHERE clauses. Key `WHERE` receives all clauses inside a `{...}` block.

To use `select` with the `where` clause you can use the following expression:
* `SELECT * FROM my_entity WHERE {<clauses>}` 

#### Example 1:
Example Request:
```sql
SELECT * FROM test_entity 
WHERE {
    ?* test_entity:age ?age, 
    ?* test_entity:race ?race, 
    (> ?age 25), 
    (in ?race "Black" "brown"),
}
```

Example response:
```rust
{
 "08824242-8098-4253-a384-987ff8d78c7d": {
  "race": String("Black"),
  "origin": String("Africa"),
  "age": Integer(34),
  "name": String("Diego F"),
 },
 "ea3228d0-0164-453b-bae2-f726c4a9b979": {
  "origin": String("Bukhara"),
  "race": String("brown"),
  "age": Integer(33),
  "name": String("julia naomi"),
 },
}
```
#### Example 2:
Example request:
```sql
SELECT * FROM test_entity 
WHERE {
    ?* test_entity:age ?age, 
    (between ?age 18 27),
}
```

Example response:
```rust
{
 "475fab3b-b023-4f6c-a18e-7e8b25b84a28": {
  "age": Integer(25),
  "name": String("Otavio"),
  "origin": String("Brasil"),
  "race": String("Multiracial"),
 },
}
```

#### TODOs:
- [ ] Support temporality for where clause

