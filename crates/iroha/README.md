# Iroha Client

This is the Iroha 2 client library crate. With it you can build your own client applications to communicate with peers in an Iroha 2 network via HTTP/WebSocket.

Follow the [Iroha 2 tutorial](https://docs.iroha.tech/guide/tutorials/rust.html) for instructions on how to set up, configure, and use the Iroha 2 client and client library.

## Features

* Submit one or several Iroha Special Instructions (ISI) as a Transaction to Iroha Peer
* Request data based on Iroha Queries from a Peer

## Setup

**Requirements:** a working [Rust toolchain](https://www.rust-lang.org/learn/get-started) (version 1.60), installed and configured.

Add the following to the manifest file of your Rust project:

```toml
iroha = { git = "https://github.com/hyperledger-iroha/iroha" }
```

## Examples

We highly recommend looking at the sample [`iroha`](../iroha_cli) implementation binary as well as our [tutorial](https://docs.iroha.tech/guide/tutorials/rust.html) for more examples and explanations.
