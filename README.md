# WooriDB (to be defined)

WooriDB is an immutable time serial database.


## Benchmark

### with pure async
create_entity           time:   [15.313 ms 15.437 ms 15.566 ms]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe

### with actors
create_entity           time:   [15.504 ms 15.568 ms 15.634 ms]
                        change: [-0.0925% +0.8535% +1.7479%] (p = 0.07 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
