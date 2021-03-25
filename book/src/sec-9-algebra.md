# Relation Algebra Functions

WooriDB has some support to relation algebra functions as well as auxiliary functions to relation algebra. They are:
- [`GROUP BY`](#group-by)
- [`ORDER BY`](#order-by)
- [`DEDUP`](#dedup)
- [`LIMIT`](#limit-and-offset)
- [`OFFSET`](#limit-and-offset)
- [`COUNT`](#count)

This functions are only supported by the following select queries:
- `SELECT */#{...} FROM  tree_key_name`
- `SELECT */#{...} FROM  tree_key_name WHERE {...}`
- `SELECT */#{...} FROM  tree_key_name IDS IN #{...}`

##  `GROUP BY`
This groups the responses of the select query in the following type `HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>` (for `group by` associated with `order by` the type is `HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>`). So the query `SELECT * FROM key GROUP BY c` for the following 6 entities:

```rust
{a: 123, b: 12.3,}
{a: 235, b: 12.3, c: 'c',}
{a: 235, b: 12.3, c: 'd',}
{a: 25, b: 12.3, c: 'c',}
{a: 475, b: 12.3, c: 'd',}
{a: 295, b: 12.3, c: 'r',}
```

Will produce the response:
```rust
{
    "Char(\'r\')": {<Uuid6>: {a: 295, b: 12.3, c: 'r',},} ,
    "Char(\'c\')": {<Uuid2>: {a: 235, b: 12.3, c: 'c',}, <Uuid4>: {a: 25, b: 12.3, c: 'c',},},
    "Char(\'d\')": {<Uuid3>: {a: 235, b: 12.3, c: 'd',}, <Uuid5>: {a: 475, b: 12.3, c: 'd',},},
    "Nil": {<Uuid1>: {a: 123, b: 12.3,},},
}
```

* Note that the Hash of the type is a `String` containing a `wql::Types`.

##  `ORDER BY`
This functions orders the response of the query by the value of a key. The key-value can be ordered by `:asc` or `:desc`. So the query `SELECT * FROM key ORDER BY a :asc` will return a `Vec<(Uuid, HashMap<String, Types>)>` for the following 6 entities:

```rust
{a: 123, b: 12.3,}
{a: 235, b: 12.3, c: 'c',}
{a: 235, b: 12.3, c: 'd',}
{a: 25, b: 12.3, c: 'c',}
{a: 475, b: 12.3, c: 'd',}
{a: 295, b: 12.3, c: 'r',}
```

Will produce the response:
```rust
[
    (<Uuid4>, {a: 25, b: 12.3, c: 'c',}),
    (<Uuid1>, {a: 123, b: 12.3,}),
    (<Uuid2>, {a: 235, b: 12.3, c: 'c',}),
    (<Uuid3>, {a: 235, b: 12.3, c: 'd',}),
    (<Uuid6>, {a: 295, b: 12.3, c: 'r',}),
    (<Uuid5>, {a: 475, b: 12.3, c: 'd',}),
]
```

- [ ] [Order By with multiple arguments](https://github.com/naomijub/wooridb/issues/101). The problem here is how to have multiple `.and_then(...)` alter the `partial_cmp`.

##  `DEDUP`

This functios is capable of removing duplicates key-values in responses. By using `SELECT * FROM key DEDUP a` for the following 6 entities:

```rust
{a: 123, b: 12.3,}
{a: 235, b: 12.3, c: 'c',}
{a: 235, b: 12.3, c: 'd',}
{a: 25, b: 12.3, c: 'c',}
{a: 475, b: 12.3, c: 'd',}
{a: 295, b: 12.3, c: 'r',}
```

We would have as a result something like:

```rust
{
    <Uuid1>: {a: 123, b: 12.3,},
    <Uuid2>: {a: 235, b: 12.3, c: 'c',},
    <Uuid3>: {a: 25, b: 12.3, c: 'c',},
    <Uuid4>: {a: 475, b: 12.3, c: 'd',},
    <Uuid5>: {a: 295, b: 12.3, c: 'r',},
}
```

Also it is possible to eliminate `Nil` and `Types::Nil` values for a `DEDUP` key. This is done by calling the function `NIL()` (It needs to be **UPPERCASE**) with the key used for the `DEDUP`. So for the previous data the response for the query `SELECT * FROM key DEDUP NIL(c)` would be:

```rust
{
    <Uuid2>: {a: 235, b: 12.3, c: 'c',},
    <Uuid4>: {a: 475, b: 12.3, c: 'd',},
    <Uuid5>: {a: 295, b: 12.3, c: 'r',},
}
```

## `LIMIT` and `OFFSET`

The functions `LIMIT` and `OFFSET` expect a positive integer as argument, this means that if you define `LIMIT 10` and `OFFSET 5` you will skip the first 5 elements from the tree and take only the next 10 elements. `LIMIT` and `OFFSET` are also appended to the end of the select query such that `SELECT * FROM key LIMIT 100 OFFSET 300`.

##  `COUNT`

This function is appended to the end of a select query and it will return the count for entities found by that select. So a query like `SELECT * FROM key WHERE {...} COUNT` will return the responses for select where as well as the count of entities found in that select. The aswer will be in the following structure:

```rust
(
    response: { "map containing the response for the query" },
    count: usize,
)
```