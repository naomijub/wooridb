FROM rust:latest
RUN apt-get update && apt-get install make

ADD https://github.com/naomijub/wooridb/archive/0.1.6.tar.gz /
RUN tar -zxvf 0.1.6.tar.gz
WORKDIR /wooridb-0.1.6
RUN rm -rf book/ woori-db/data/ benches/ data/*.txt

EXPOSE 1438

ENTRYPOINT [ "make" ]