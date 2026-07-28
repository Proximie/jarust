# Jarust &emsp; [![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/jarust.svg
[crates.io]: https://crates.io/crates/jarust

Jarust is a memory safe and high-performance Rust adapter for [Janus WebRTC server](https://github.com/meetecho/janus-gateway).
Inspired by [Janode](https://github.com/meetecho/janode), jarust offers similar functionalities but it's designed
to be customizable, for example, you could use the built-in WebSocket transport or provide your own RabbitMQ transport implementation.

The library wraps the Janus core API and some of the most popular plugins APIs.

## Plugins

The supported Janus plugins currently are:

- [x] EchoTest
- [x] AudioBridge
- [x] Streaming ([Experimental](./CONTRIBUTING.md#experimental))
- [x] VideoRoom ([Experimental](./CONTRIBUTING.md#experimental))

## Interfaces

The supported interfaces are:

- [x] WebSocket
- [x] Restful
- [ ] MQTT
- [ ] RabbitMQ
- [ ] Nanomsg

## APIs

- [x] Client API
- [ ] Admin/Monitor API

## Examples

To run the examples first you have to launch the janus server.

```sh
docker compose up -d
```

Examples are available in [`jarust` crate](./jarust/examples/)
