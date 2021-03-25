# using Json

WoorDB is able to support Json requests and responses by using the flag `features`. To execute WooriDB with `json` feature enable execute:

- `Make json`, or;
- `cargo run --manifest-path woori-db/Cargo.toml --release --features json`

> * Remember that Json doesn't have trailing commas while ron has them.

## Example request:
For `/auth/createUser`.

```json
{
    "admin_id": "your_admin", 
    "admin_password": "your_password", 
    "user_info": {
        "user_password": "my_password",
        "role": ["User"]
    }
}
```

## Example response:
For a `SELECT * FROM entity`.

```json
{
 "38d52c95-b6f6-403a-a0b2-447b8fa15784": {
  "a": Integer(123),
  "tx_time": DateTime("2021-03-24T23:56:45.179008791Z"),
 },
}
```

