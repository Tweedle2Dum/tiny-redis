# tiny-redis

A tiny, educational Redis-like in-memory datastore built in Rust.  
The goal of this project is to understand how Redis works under the hood — protocols, command handling, in-memory storage, concurrency, and networking — without the complexity of the full Redis codebase.

This project **is not a full Redis clone**, but a minimal implementation that you can extend to learn systems programming, network servers, and Rust async patterns.

---

## ✨ Features (current)

- Simple in-memory key–value store  
- Basic Redis-like commands (`SET`, `GET`, `DEL`, etc.)  
- Minimal TCP server  
- Basic text-based protocol (RESP-ish)  
- Small, readable codebase meant for learning  

---

## 🚧 Roadmap (next steps)

tiny-redis is intentionally small — but here’s what’s coming next:

- [ ] More commands (`EXISTS`, `INCR`, hashes/lists)  
- [ ] Proper RESP protocol support  
- [ ] Multiple clients & concurrency  
- [ ] Async TCP server (Tokio)  
- [ ] TTL / key expiration  
- [ ] Persistence (AOF or snapshot)  
- [ ] Benchmarks + CI  
- [ ] Published Rust crate  

---

## 🧰 Getting Started

### **Prerequisites**

- Rust (stable)
- Cargo

### **Clone the project**

```bash
git clone https://github.com/Tweedle2Dum/tiny-redis
cd tiny-redis
Run the server
bash
Copy code
cargo run
This starts a small Redis-like TCP server on port 6379 (by default).

Connect using redis-cli
bash
Copy code
redis-cli -p 6379
Example:

sql
Copy code
SET hello world
GET hello
📦 Project Structure
bash
Copy code
tiny-redis/
│
├── src/
│   ├── main.rs          # Server entry point
│   ├── protocol.rs      # Parsing commands / minimal RESP
│   ├── store.rs         # In-memory key-value store
│   └── server.rs        # TCP listener & client management
│
└── README.md
🧑‍💻 Example Commands
Using netcat / raw TCP:

bash
Copy code
nc localhost 6379
Then:

powershell
Copy code
SET foo bar
GET foo
DEL foo
🎯 Purpose
Redis is powerful, but its C codebase is large and optimized for production.
tiny-redis exists so developers can clearly understand:

How a TCP server is structured

How commands & protocols are parsed

How an in-memory store works internally

How state is shared between connections

How async & concurrency models apply to databases

🤝 Contributing
PRs are welcome — especially around:

Protocol parsing

New commands

Documentation

Examples

Refactoring or cleanup

Feel free to open an issue if you'd like help contributing.

📄 License
MIT License — free to use, modify, distribute, and learn from.