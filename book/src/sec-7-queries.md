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