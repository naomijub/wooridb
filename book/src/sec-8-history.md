# Entity History

Entity history is one of the main features of WooriDB. It receives an `entity_key` name and an `entity_id` which will return the whole history of `(DateTime<Utc>, entity_map)` for the `entity_id` in the entity tree for key `entity_key`. This is done by sending a `POST` request to endpoint `<ip>:1438/entity-history`. An example request would be `curl -X POST -H "Content-Type: application/wql" <ip>:1438/entity-history -d '(entity_key: "entity_tree_key", entity_id: "<some-Uuid>",)'`. In `release mode` it is necessary to use header `Authorization: Bearer <your session token>` for this endpoint. 

Example request: 
```ron
(entity_key: "entity_tree_key", entity_id: "dc3069e7-2a22-4fbc-ae05-f78a807239c0",)
``` 

Example response:
```rust
{
"2021-03-02T05:00:19.813514240Z": {
    "a": Integer(123),
    "b": Float(12.3),
    "tx_time": DateTime("2021-03-02T05:00:19.813514240Z"),
},
"2021-03-02T05:00:19.816357939Z": {
    "b": Float(12.3),
    "a": Integer(123),
    "tx_time": DateTime("2021-03-02T05:00:19.816357939Z"),
},
"2021-03-02T05:00:19.817189987Z": {
    "b": Float(12.3),
    "c": Boolean(true),
    "a": Integer(34),
    "tx_time": DateTime("2021-03-02T05:00:19.817189987Z"),
},
"2021-03-02T05:00:19.818031113Z": {
    "b": Float(12.3),
    "a": Integer(321),
    "c": Char('h'),
    "tx_time": DateTime("2021-03-02T05:00:19.818031113Z"),
},}
```

* This response is considering the following events:
1. `INSERT {a: 123, b: 12.3,} INTO entity_tree_key`.
2. `UPDATE entity_tree_key SET {{a: 12, c: Nil,}} INTO dc3069e7-2a22-4fbc-ae05-f78a807239c0`. Note that this event does not appear due to DELETE.
3. `Delete dc3069e7-2a22-4fbc-ae05-f78a807239c0 FROM entity_tree_key`. Entity map state is the same as the previous state, therefore "2021-03-02T05:00:19.813514240Z" and "2021-03-02T05:00:19.816357939Z" have equal content.
4. `UPDATE entity_tree_key SET {{a: 34, c: true,}} INTO dc3069e7-2a22-4fbc-ae05-f78a807239c0`.
5. `UPDATE entity_tree_key SET {{a: 321, c: 'h',}} INTO dc3069e7-2a22-4fbc-ae05-f78a807239c0`.

## Entity history with time ranges

There are two extra parameters that can be used with `entity-history`, they are `start_datetime` and `end_datetime`. Both parameters are optional and if they are present they will define the time limits of the query. `start_datetime` is the beginning of the time range query while `end_datetime` is the ending of the time range query. If we used `start_datetime` and `end_datetime` for the previous example as `curl -X POST -H "Content-Type: application/wql" <ip>:1438/entity-history -d '(entity_key: "entity_tree_key", entity_id: "<some-Uuid>", start_datetime: Some("2021-03-02T05:00:19.816357937Z"), end_datetime: Some("2021-03-02T05:00:19.817189988Z"),)'` we would have the following result:

Example request: 
```ron
(
    entity_key: "entity_tree_key", 
    entity_id: "dc3069e7-2a22-4fbc-ae05-f78a807239c0",  
    start_datetime: Some("2021-03-02T05:00:19.816357937Z"), 
    end_datetime: Some("2021-03-02T05:00:19.817189988Z"),
)
``` 

Example response:
```rust
{
"2021-03-02T05:00:19.816357939Z": {
    "b": Float(12.3),
    "a": Integer(123),
    "tx_time": DateTime("2021-03-02T05:00:19.816357939Z"),
},
"2021-03-02T05:00:19.817189987Z": {
    "b": Float(12.3),
    "c": Boolean(true),
    "a": Integer(34),
    "tx_time": DateTime("2021-03-02T05:00:19.817189987Z"),
},}
```