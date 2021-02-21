# Introduction

WooriDB is a general purpose (**EXPERIMENTAL**) time serial database, which means it contains all entities registries indexed by DateTime and `Uuid`. It is schemaless, deep key-value storage and uses its own query syntax that is similar to SparQL and Crux's Datalog. 

## Name origin
`Woori` means `our`, in Korean, and although I developed this DB initially alone, it is in my culture and my hunsband's culture to call everything that is done for our communities and by our communities **ours**. I chose *Woori* instead of *shelanu* or *bizdin* because it easier to pronunce.

## Project inspirations
- [Crux](https://github.com/juxt/crux) a general purpose bitemporal graph query database with support for SQL and Datalog. It was the ideallogical source of WooriDB. I had developed a rust client called [Transistor](https://github.com/naomijub/transistor) which gave me the basis of what I wanted to have in WooriDB.
- [Datomic](https://www.datomic.com/) is a transactional database with a flexible data model, temporality and rich queries. I worked with Datomic at [Nubank](https://nubank.com.br/sobre-nos/) and it is the reason I found Juxt/Crux. 
- [Prometheus](https://github.com/prometheus/prometheus) An open-source monitoring system with a dimensional data model, flexible query language, efficient **time series database** and modern alerting approach.
- [SparQL](https://en.wikipedia.org/wiki/SPARQL) SPARQL is a query language for  RDF graph databases, it is flexible enough for query information based on datetime indexes. 
- [Database Internals](https://www.amazon.com.br/Database-Internals-Alex-Petrov/dp/1492040347/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Database+Internals%3A&qid=1612831621&sr=8-1)
- [Database System Concept](https://www.amazon.com.br/dp/B073MPV4YC/ref=dp-kindle-redirect?_encoding=UTF8&btkr=1)
- [Designing Data Intensive Application](https://www.amazon.com.br/Designing-Data-Intensive-Applications-Reliable-Maintainable-ebook/dp/B06XPJML5D/ref=sr_1_1?__mk_pt_BR=%C3%85M%C3%85%C5%BD%C3%95%C3%91&dchild=1&keywords=Designing+Data%E2%80%93Intensive+Applications&qid=1612831724&s=books&sr=1-1)
- Professor [Andy Pavlo](http://www.cs.cmu.edu/~pavlo/) Database Design Course. 

## Naming conventios:
- Entity Tree is similar to SQL table, it is the data structure that contains all ids and entities map relations.
- Entity ID is the id of an entity inside Entity tree.
- Entity map is the content of and entity associated with the entity id.