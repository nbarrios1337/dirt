# `dirt` - DNS Information Resolving Tool

An in-progress DNS resolver written in Rust, based on [Julia Evan's "Implement DNS in a weekend" guide](https://implement-dns.wizardzines.com/index.html).

Meant as a foray into networking, I intend for this project to be my own inplementation of the DNS standards, with an eye towards handling some of `dig`'s simpler functions.

## Current Features

- [x] Query creation
- [x] Header and question parsing
- [x] recursive resolving
- [x] type-dependent record parsing (A and NS types)

## TODO / Potential Features

- [ ] caching (databases?)
- [ ] asynchronous queries
- [ ] request/response multitasking
