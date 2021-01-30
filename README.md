# WooriDB (to be defined)

WooriDB is an immutable time serial database.

## Transactions:

### Parser
- [ ] Query language parser

### Transactions by Query
- [x] Create entity
- [ ] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities

### Transactions by Endpoint
- [ ] Create entity
- [ ] Insert entity
- [ ] Update entity
- [ ] Delete entity
- [ ] Evict entity
- [ ] Select entities


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
time:   [15.504 ms 15.568 ms 15.634 ms]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
```