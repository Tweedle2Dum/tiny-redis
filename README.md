# tiny-redis

A minimal, educational Redis-like in-memory datastore built in Rust.

This project exists to demystify how Redis works under the hood—covering protocols, command handling, in-memory storage, and TCP networking—without the overwhelming complexity of the full Redis codebase.

**Not a production Redis replacement**, but a learning-focused implementation you can read, understand, and extend.

---

## Why tiny-redis?

Redis is powerful but has decades of optimizations and a large C codebase. **tiny-redis** strips away the complexity to reveal the core concepts:

- How TCP servers handle multiple clients
- How commands are parsed from byte streams
- How in-memory key-value stores work
- How async patterns enable concurrency

Perfect for systems programming students, Rustaceans, or anyone curious about database internals.

---

## Features

- ✅ In-memory key-value store with `HashMap`
- ✅ RESP (Redis Serialization Protocol) parser
- ✅ Basic commands: `PING`, `ECHO`, `SET`, `GET`, `DEL`
- ✅ TCP server with connection handling
- ✅ Incremental command parsing (handles partial/pipelined commands)
- ✅ Clean, readable codebase (~400 lines)

---

## Getting Started

### Prerequisites

- Rust (stable channel)
- Cargo

### Installation

```bash
git clone https://github.com/Tweedle2Dum/tiny-redis
cd tiny-redis
cargo build --release
```

### Run the Server

```bash
cargo run
```

The server starts on `127.0.0.1:6379` by default.

### Connect with `redis-cli`

```bash
redis-cli -p 6379
```

Try these commands:

```redis
PING
# => PONG

SET greeting "Hello, World!"
# => OK

GET greeting
# => "Hello, World!"

ECHO "testing"
# => "testing"

DEL greeting
# => (integer) 1
```

### Connect with `netcat`

```bash
nc localhost 6379
```

Send raw RESP commands:

```
*1\r\n$4\r\nPING\r\n
*3\r\n$3\r\nSET\r\n$3\r\nkey\r\n$5\r\nvalue\r\n
*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n
```

---

## Project Structure

```
tiny-redis/
├── src/
│   ├── main.rs         # Entry point, server initialization
│   ├── server.rs       # TCP listener & connection handler
│   ├── parser.rs       # RESP protocol parser
│   ├── executor.rs     # Command parsing & execution
│   └── db/
│       └── mod.rs      # In-memory HashMap store
├── Cargo.toml
└── README.md
```

### Module Overview

- **`server`**: Manages TCP connections, reads bytes, delegates to handler
- **`parser`**: Parses RESP protocol (arrays, bulk strings, integers)
- **`executor`**: Converts parsed RESP into commands, executes them
- **`db`**: Simple `HashMap<String, String>` wrapper

---

## Architecture

```
Client (redis-cli)
      ↓
TCP Connection
      ↓
[server.rs] Read bytes into buffer
      ↓
[parser.rs] Parse RESP → RespValue
      ↓
[executor.rs] Convert to Command enum → Execute
      ↓
[db.rs] Read/write HashMap
      ↓
[executor.rs] Generate RESP response
      ↓
[server.rs] Write response to client
```

---

## Roadmap

This is an intentionally minimal implementation. Here's what's next:

- [ ] **More commands**: `EXISTS`, `INCR`, `DECR`, `APPEND`, `STRLEN`
- [ ] **Data structures**: Hashes, Lists, Sets
- [ ] **Concurrency**: Async I/O with Tokio, multi-threaded executor
- [ ] **TTL/Expiration**: Time-based key expiry
- [ ] **Persistence**: Append-only file (AOF) or snapshotting
- [ ] **Transactions**: `MULTI`/`EXEC` support
- [ ] **Pub/Sub**: Basic message channels
- [ ] **Benchmarks**: Performance testing & optimization
- [ ] **Proper error handling**: Better RESP error responses

---

## Contributing

PRs are welcome! Areas where help would be great:

- Implementing new Redis commands
- Adding more tests
- Documentation improvements
- Performance optimizations
- Refactoring for clarity

**Before submitting**: Run `cargo test` and `cargo clippy`.

---

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover:
- RESP protocol parsing (arrays, strings, integers)
- Command parsing and execution
- Incomplete/partial command handling
- Multi-command pipelining

---

## Learning Resources

Want to dive deeper?

- [Redis Protocol (RESP) Specification](https://redis.io/docs/reference/protocol-spec/)
- [Build Your Own Redis (codecrafters.io)](https://codecrafters.io/challenges/redis)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) (for async version)
- [The Rust Book](https://doc.rust-lang.org/book/)

---

## License

MIT License. Free to use, modify, and learn from.

---

## Acknowledgments

Inspired by:
- **Redis** by Salvatore Sanfilippo
- **mini-redis** (Tokio's tutorial project)
- The Rust community's commitment to teaching systems programming

---

**Built with ❤️ and Rust**