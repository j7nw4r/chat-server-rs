# chat-server-rs

`chat-server-rs` is a basic live chat server implemented in Rust.

## Running

To run: 

```bash
cargo run
```

When running chat-server-rs exposes one websocket endpoint at `http://localhost:23234/ws/chat`.

To chat send a first message with a json: 

```json
{
    "user": "<user_id>",
    "channel": "<channel_id>"
}
```

This will connect you to the requested channel as the given user. You will then receive all messages passed to that channel.

Messages are just strings.