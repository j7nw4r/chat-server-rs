# chat-server-rs

`chat-server-rs` is a basic live chat server implemented in Rust.

## Running

To run: 

```bash
cargo run
```

When running chat-server-rs exposes one websocket endpoint at `http://localhost:23234/ws/chat`.

## Sending messages

Messages are formed as follows:

```json
{
    "type": "Message",
    "user": "<user_id>",
    "content": "<content string>"
}

```

## Testing

Postman has a good websocket client for testing.