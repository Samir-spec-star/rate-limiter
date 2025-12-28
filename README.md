# Rate Limiter
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Axum](https://img.shields.io/badge/Axum-0.8-purple)](https://github.com/tokio-rs/axum)
[![Redis](https://img.shields.io/badge/Redis-Supported-red?logo=redis)](https://redis.io/)
[![Tokio](https://img.shields.io/badge/Tokio-Async-blue)](https://tokio.rs/)
> A production-grade, async rate limiting library built in Rust with multiple algorithms, pluggable storage backends, and HTTP middleware support.
---

## Table of Contents
- [Features](#-features)
- [Why This Project?](#-why-this-project)
- [Installation](#-installation)
- [Dependencies](#-dependencies)
- [Quick Start](#-quick-start)
- [Performance Benchmarks](#-performance-benchmarks)
- 
---

## Features 

| Feature | Description |
|---------|-------------|
| **Token Bucket Algorithm** | Smooth rate limiting with burst capacity support |
| **Sliding Window Algorithm** | Accurate request counting with weighted averages |
| **In-Memory Storage** | High-performance storage using DashMap (lock-free reads) |
| **Redis Storage** | Distributed rate limiting for horizontal scaling |
| **Axum Middleware** | Production-ready HTTP middleware with standard headers |
| **Fully Async** | Built on Tokio for high-concurrency workloads |
| **Thread-Safe** | Safe concurrent access with `Send + Sync` guarantees |
| **Per-Client Limiting** | Rate limit by IP, API key, or custom identifiers |
| **Standard Headers** | `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `Retry-After` |
| **Graceful Degradation** | Falls back to memory if Redis is unavailable |
---

## Why this project?
Rate limiting is essential for:
- **Preventing API Abuse** - Stop malicious users from overwhelming your server
- **Fair Resource Allocation** - Ensure all users get equal access
- **Cost Control** - Limit expensive operations (database queries, AI calls)
- **DDos Protection** - First line of defence against denial-of-service attacks
- **Compliance** - Meet SLA requirements for API usage

This Library provides **production-ready** rate limiting that you can drop into any Rust web application.

---

## Installation 

### From Source 

```bash
git clone https://github.com/Samir-spec-star/rate-limiter.git
```
```bash
cd rate_limiter
```
```bash
cargo build --release
```
## Dependencies 
| Crate | version | Purpose |
|-------|---------|---------|
|**axum** | 0.8.7 | HTTP framework |
|**tokio | 1.48.0 | Async runtime |
|**dashmap** | 6.1.0 |Concurrent HasHMap |
|**redis** | 1.0.0 | Redis client |
|**thiserror** | 2.0.17 | Error Handling |
|**async-trait** | 0.1.89 | Async trait support|
|**Tower** | 0.5.2 | MIddleware abstraction |
|**Tower-http** | 0.6.8 | HTTP middleware |
---

## Quick Start
```bash
cargo run --release
```

## performnace bench mark 
![Screenshot 2025-12-26 112116](https://github.com/user-attachments/assets/eedc1e66-38dc-4aef-b2da-4d27588fcb9b)![4ff0b2c9-db63-478c-8c42-069d09c3f54b](https://github.com/user-attachments/assets/9e50e914-3981-46a0-b41f-4ea1fecb68a7)





