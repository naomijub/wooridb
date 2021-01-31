# WooriDB (to be defined)

WooriDB is an immutable time serial database.

## Installation

- `make release` at `./woori-db`; or
- `cargo run --release`
- `cargo run --release` at root or at `./woori-db`;

## Usage



## Transactions:

### Parser
- [ ] Query language parser

### Transactions by Query
- [x] Create entity
- [x] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities

<!-- ### Transactions by Endpoint
- [ ] Create entity
- [ ] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities ???? -->


### SELECT = Functions that could be implemented from Relation Algebra:
- [ ] Select
- [ ] Projection
- [ ] Union
- [ ] Intersection
- [ ] Difference (SQL's EXCEPT?)
- [ ] Join
- [ ] Product (SQL's CROSS JOIN?)
- [ ] Rename
- [ ] Assign
- [ ] Dedup
- [ ] Sort
- [ ] Aggregate
- [ ] Division

## Benchmark

* `create_entity`
```
time:  [15.443 ms 15.496 ms 15.547 ms]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
```

* `insert_entity`
```
time:   [15.623 ms 15.661 ms 15.699 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
```