# Error Messages

Woori DB has a variety of Error messages, from WQL when parsing the transaction or the query to some internal database errors. Here are listed expected messages:

## WooriDB Errors:

### Example errors as response:
```rust
(
 error_type: "CheckNonEncryptedKeys",
 error_message: "CHECK can only verify encrypted keys: [\"ssn\"]",
)
```

### Error types:
- `Io(io::Error)` - Failed to read or write file.
- `QueryFormat(String)` - WQL error.
- `EntityAlreadyCreated(<entity_name>)` - Entity `entity_name` already created in the database.
- `EntityNotCreated(<entity_name>)` - tx or query at entity tree does not contain key `entity_name`.
- `EntityNotCreatedWithUniqueness(<entity_name>)` - this error only occurs if a failed write to the bank happened. Migration to fix data inconsistency may be needed.
- `Serialization(ron::Error)` - Ron serialization error in wql context.
- `UuidNotCreatedForEntity(<entity_name>, Uuid)` - If you try to `UPDATE/DELETE/EVICT/SELECT` an Uuid that `entity_name` does not contain.
- `FailedToParseState` - Failed to read log file state/entity map.
- `FailedToParseRegistry` - Failed to read log file registry information
- `UnknownCondition` - `MATCH UPDATE` has an unknown condition (`==`, `>=`, `<`, etc).
- `FailedMatchCondition` - `MATCH UPDATE` internal service error while processing conditions.
- `DuplicatedUnique(<entity_name>, <entity_map_key>, Types)` - This means that `Types` is already present at `entity_map_key` for entity tree `entity_name`.
- `SelectBadRequest` - Select operation done at `/wql/tx`. Select operations are done at `/wql/query`.
- `NonSelectQuery` - Tx operation done at `/wql/query`. Tx operations are done at `/wql/tx`.
- `ActixMailbox(MailboxError)` - Internal server error meaning that some actor mailbox crashed.
- `LockData` - Failed to get a lock on Data
- `Ron(ron::Error)` - Ron serialization error that occured in user creation.
- `InvalidUuid(uuid::Error)` - Uuid could not be parsed.
- `UpdateContentEncryptKeys(Vec<keys>)` - `UPDATE CONTENT` cannot update encrypted `keys`.
- `CheckNonEncryptedKeys(Vec<keys>)` - Cannot `CHECK` non encrypted `keys`.
- `DateTimeParse(chrono::ParseError)` - failed to parse input `DateTime<UTC>`.
- `FailedToParseDate` - failed to parse log file saved date.
- `AdminNotConfigured` - Admin is not configured at release mode, please check [auth section](./sec-5-auth.md) for more info.
- `AuthBadRequest` - Authentication & Authorization error.
- `FailedToCreateUser` - Failed to create new user.
- `Unknown` - Unknown error.
- `KeyTxTimeNotAllowed` - the key `tx_time` is not allowed in entities map for inserts and updates.

## WQL Parsing
- `Query symbol error`: 
    - "Symbol `{symbol_name}` not implemented". Symbol name are the start of the query, like `SELECT, CHECK, CREATE, ISNERT`.

- `Keyword error`:
    - `CREATE`: Keyword ENTITY is required for CREATE"
        - "Correct wording is ENCRYPT" for `CREATE ENTITY ENCRYPT`
        - "Correct wording is UNIQUES" for `CREATE ENTITY UNIQUES`.
    - `UPDATE`: "UPDATE type is required after entity. Keywords are SET or CONTENT"
        - "Keyword INTO is required for UPDATE"
        - "Keyword INTO is required for MATCH UPDATE"
        - "UPDATE keyword is required for MATCH UPDATE"
        - "MATCH UPDATE type is required after entity. Keyword is SET". Use `SET` as update type in `MATCH`.
    - `SELECT/CHECK`: "Keyword FROM is required for CHECK"
        - "Keyword FROM is required for SELECT"
        - "WHEN not allowed after IDS IN"
        - "Keyword AT is required after WHEN"
        - "Keyword IN is required after IDS to define a set of uuids"
        - "Keyword ID/IDS is required to set an uuid in SELECT".
    - `EVICT/DELETE/INSERT`: "Keyword FROM is required for DELETE"
        - "Keyword INTO is required for INSERT"
        - "Keyword FROM is required to EVICT an UUID".

- `Argument format error`:
    - "Arguments set should start with `#{` and end with `}`"
    - "Entity map should start with `{` and end with `}`" occurs mostly with `INSERT and UPDATE`.
    - "Field ID must be a UUID v4"
    - `SELECT`: "Encrypted arguments cannot be set to UNIQUE"
        -   "SELECT expression should be followed by `*` for ALL keys or `#{key_names...}` for some keys" and "SELECT arguments set should start with `#{` and end with `}`"
        -   "Uuids in `IDS IN` are reuired to be inside a `#{` and `}`"
        -   "START date and END date should be the same date."
        -   "WHERE clauses must be contained inside `{...}`"

- `Required content`:
    - "MATCH requires ALL or ANY symbols". It is necessary to include `ANY` or `ALL` conditions after `MATCH` keyword.
    - "Entity UUID is required for DELETE"
    - `Entity name`: 
        - "Entity name is required after FROM"
        - "Entity name is required for SELECT"
        - "Entity name is required after INTO"
        - "Entity name is required for UPDATE"
        - "Entity name is required for EVICT"
        - "Entity name is required for MATCH UPDATE"

- `Parse error`:
    - "Couldn't create uuid from {some-text}. Error: {Uuid::Error}"
    - "Couldn't parse UPDATE query". `SET` or `CONTENT` were not found after `UPDATE`
    - "Couldn't parse MATCH UPDATE query"
    - "Entity name cannot contain `-`". Entity name should contain only alphanumeric and `_` chars.
    - "Key must be an alphanumeric value" or contain `_`.
    - "Hash cannot be hashed" and "Nil cannot be hashed". `Types::Hash` and `Types::Nil` cannot be hashed.
    - "Not able to parse match argument". Match condition has wrong argument type.
    - "Unidentified Match Condition". Could not identify match condition.
    - "Entity HashMap could not be created"
    - "Value Type could not be created from {some value}". Could not create `Types` from `some value`.
    - "WHERE clause cannot be empty"
