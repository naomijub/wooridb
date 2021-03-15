FROM rust:latest
RUN apt-get update && apt-get install make

ADD https://github.com/naomijub/wooridb/archive/0.1.4.tar.gz /
RUN tar -zxvf 0.1.4.tar.gz
WORKDIR /wooridb-0.1.4
RUN rm -rf book/ woori-db/data/ benches/ data/*.txt

EXPOSE 1438

ENTRYPOINT [ "make" ]