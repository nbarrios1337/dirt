# `dirt` - DNS Information Resolving Tool

An in-progress DNS resolver written in Rust, based on [Julia Evan's "Implement DNS in a weekend" guide](https://implement-dns.wizardzines.com/index.html).

Meant as a foray into networking, I intend for this project to be my own implementation of the DNS standards, with an eye towards handling some of `dig`'s simpler functions.

## Current Features

- [x] Query creation
- [x] Header and question parsing
- [x] recursive resolving
- [x] type-dependent record parsing (A and NS types)
- [x] IPv6 querying support

## TODO / Potential Features

- [ ] caching (databases?)
  - [ ] resolver would want file persistence across runs
  - [ ] server would likely keep cache in-memory and with file backing
- [ ] asynchronous queries
  - [ ] this is likely server specific, unless we create a multi-client resolver service
- [ ] request/response multitasking
  - [ ] applicable to resolver service + server
- [ ] Library-Binary separation
  - [ ] library that provides definitions of common DNS data structures
  - [ ] simple recursive resolver binary
  - [ ] multi-client resolver service binary
  - [ ] multi-client DNS server binary
- [ ] TCP support
- [ ] More command-line arguments
  - [ ] recursion desired, authoritative answer requested, etc.
