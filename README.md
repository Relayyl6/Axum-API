# Rust Axum + PostgreSQL (Docker + WSL)

A fully containerized backend API built with **Rust (Axum)** and **PostgreSQL**, designed to run smoothly inside **WSL using Docker Engine**.

This project demonstrates:

* REST API with Axum
* PostgreSQL integration using SQLx
* Docker multi-stage builds
* Clean local + containerized workflows

---

# Tech Stack

* **Rust** (Axum framework)
* **PostgreSQL 17**
* **SQLx**
* **Docker + Docker Compose**
* **WSL (Ubuntu/Debian)**

---

# 1. Docker Setup (WSL)

> Skip if Docker already works (`docker ps` works without sudo)

## A. Remove old Docker versions

```bash
sudo apt-get remove docker docker-engine docker.io containerd runc
```

## B. Install Docker Engine

```bash
sudo apt-get update
sudo apt-get install ca-certificates curl gnupg

sudo install -m 0755 -d /etc/apt/keyrings

curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
| sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

sudo chmod a+r /etc/apt/keyrings/docker.gpg

echo \
"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
https://download.docker.com/linux/ubuntu \
$(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
| sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

## C. Enable Docker without sudo

```bash
sudo groupadd docker
sudo usermod -aG docker $USER
newgrp docker
```

Test:

```bash
docker ps
```

---

# 📁2. Project Structure

```bash
.
├── src/
├── migrations/
├── Dockerfile
├── compose.yaml
├── Cargo.toml
└── README.md
```

---

# 🐳3. Docker Configuration

## 🔹 compose.yaml

```yaml
services:
  app:
    container_name: simple_axum
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://postgres:<password>@db:5432/simple_db
    depends_on:
      - db

  db:
    image: postgres:17
    container_name: db
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=<password>
      - POSTGRES_DB=simple_db
    volumes:
      - pg_data:/var/lib/postgresql/data

volumes:
  pg_data:
```

### ⚠️Important Notes

* `$` must be escaped as `$$` in Docker Compose
* `DATABASE_URL` uses URL encoding internally
* DB service name = `db` (used as hostname)

---

## 🔹Dockerfile

```dockerfile
# --- Build Stage ---
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/axumlive .

EXPOSE 3000

CMD ["./axumlive"]
```

---

# ▶️ 4. Running the App

## Full Docker Mode

```bash
docker compose down -v        # clean reset
docker compose up --build
```

Check logs:

```bash
docker logs -f simple_axum
```

---

## Fast Dev Mode (Recommended)

Run DB in Docker, app locally:

```bash
docker compose up -d db
```

```bash
export DATABASE_URL="postgres://postgres:<password>@localhost:5432/simple_db"
cargo run
```

---

# 5. Database

## Enter PostgreSQL

```bash
docker exec -it db psql -U postgres -d simple_db
```

Useful commands:

```sql
\dt        -- list tables
SELECT * FROM users;
\q         -- quit
```

---

# 6. API Endpoints

### Create User

```http
POST /users
```

```json
{
  "name": "John Doe",
  "email": "john@email.com"
}
```

---

### Get All Users

```http
GET /users
```

---

### Get User by ID

```http
GET /users/{id}
```

---

### Update User

```http
PUT /users/{id}
```

---

### Delete User

```http
DELETE /users/{id}
```

---

# ⚠️ 7. Common Issues & Fixes

## ❌ Container not running

```bash
docker logs simple_axum
```

---

## ❌ Password authentication failed

Fix:

```bash
docker compose down -v
```

Then ensure:

* password matches in both services
* `$` is escaped (`$$`)

---

## ❌ No tables (`\dt`)

* migrations not running
* or container crashed early

---

## ❌ 500 Internal Server Error on insert

Cause:

* DB column mismatch

Fix:

```rust
struct User {
    id: i32,
    username: String,
    email: String,
}
```

---

## ❌ Duplicate user error

```text
duplicate key value violates unique constraint
```

Fix:

* use different username
* or return `409 CONFLICT` in API

---

# Key Lessons

* Docker containers ≠ local filesystem
* `$` must be escaped in Compose
* DB credentials must match exactly
* SQL schema must match Rust structs
* Always check logs first (`docker logs`)

---

# Future Improvements

* Input validation
* Authentication (JWT)
* Structured error responses
* Logging middleware
* Docker health checks

---

# Author

Built as part of learning:

* Rust backend development
* Docker containerization
* Database integration
