Zed
===
> Author: Marcin Szymczak

Rust game experiment.
Client <-> Server architecture

# Introduction
This is my hobby side-project which enables me to learn Rust in a fun way.

# Current status
A basic client <-> server game with no features at all.
Allows for manipulating "player" with simple controls:
* *W, S, A, D* move Up, Down, Left, Right
* *H* "holster" weapon (There is no weapon support right now, it is fictional)
* *C* change shirt color (It is synchronized, yay! :) )

# How to run
## Start a server
`cargo run --bin zed-server -- address:port`
Starts game server on provided address:port.

## Start a client and connect to a server
`cargo run --bin zed-client -- server_address:port [local_address:port]`
Start a client and attempts to connect o server of `server_address:port` by binding a local
UDP socket to a `local_address:port`.

If local address is not provided, a random one is chosen by the system.

