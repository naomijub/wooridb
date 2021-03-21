# Installation and Important Information

## Installation

To run WooriDB it is necessary to have Rust installed in the machine. There are two ways to do this:

1. Go to rustup.rs and copy the command there, for unix it is `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. Clone WooriDB and execute `make setup`.


### Executing WooriDB

- `Release mode performance`: `make release` in project root for performance optimization.
- `Release mode size`: `make run` in project root for size optimization.
- `Debug mode`: `make debug` in project root.

### Docker

You can find the latest docker image at **[naomijub/wooridb](https://hub.docker.com/repository/docker/naomijubs/wooridb)**. Currently the most stable tag is [**`beta-4`**](https://github.com/naomijub/wooridb/releases/tag/0.1.4). To execute the docker container run:

* `docker run -p 1438:1438 naomijubs/wooridb:beta-4 debug` for debug mode.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-4 run` for size optimization.
* `docker run -p 1438:1438 -e AUTH_HASHING_COST=8 -e ADMIN=your-admin-id -e ADMIN_PASSWORD=your-admin-pswd naomijubs/wooridb:beta-4 release` for performance optimization.
* All `-e/--env` can be replaced by a `--env-file path/to/your/.env`. Your `.env` file should contain the following fields:
```
HASHING_COST=16
PORT=1438
AUTH_HASHING_COST=8
ADMIN=your-admin-id
ADMIN_PASSWORD=your-admin-pswd
``` 

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