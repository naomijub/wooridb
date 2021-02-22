# Installation and Important Information

## Installation

To run WooriDB it is necessary to have Rust installed in the machine. There are two ways to do this:

1. Go to rustup.rs and copy the command there, for unix it is `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. Clone WooriDB and execute `make setup`.


### Executing WooriDB

- `Release mode`: `make run` in project root.
- `Debug mode`: `make debug` in project root.

## Important Information

* Responses are in [`RON`](https://github.com/ron-rs/ron) format.
* `BLOB` will not be supported. Check out [To BLOB or Not To BLOB: Large Object Storage in a Database or a Filesystem](https://www.microsoft.com/en-us/research/publication/to-blob-or-not-to-blob-large-object-storage-in-a-database-or-a-filesystem/).

###  Configurations
* To run the project in `debug` mode it is important to export the following environment variables `HASHING_COST, PORT`. Default values are:
```
HASHING_COST=16
PORT=1438
```

* To run the project in `release` mode it is important to export the following environment variables `HASHING_COST, PORT, AUTH_HASHING_COST, ADMIN, ADMIN_PASSWORD`. There are no default values for `AUTH_HASHING_COST, ADMIN, ADMIN_PASSWORD`.