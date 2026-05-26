# KAIST Concurrent Programming in Rust

This repository contains my implementation of concurrent data structures and libraries in Rust, from the [KAIST CS431](https://github.com/kaist-cp/cs431) course.

## Course Overview

**Understand the motivations and challenges of concurrent programming.** Learn design patterns and principles for reasoning about concurrency. Design, implement, and evaluate concurrent programs. Apply knowledge to real-world parallel systems.

## Implementations

### Core Concurrent Data Structures

1. **Arc** - Atomic Reference Counting for shared ownership across threads
   - Enables safe shared access to heap data without garbage collection
   - Foundation for building other concurrent structures

2. **BoC (Bag of Counts)** - A concurrency pattern using CownPtr
   - Demonstrates scalable concurrent counting
   - Lock-free approach to distributed counters

3. **ElimStack** - Elimination-based stack for concurrent access
   - Scales better than traditional locked stacks
   - Threads coordinate directly to eliminate push-pop pairs

4. **LinkedList** - Concurrent linked list implementation
   - Fine-grained locking and optimistic approaches
   - Handling concurrent insertions, deletions, and lookups

5. **HashTable** - Concurrent hash table structures
   - **GrowableArray**: Dynamic resizing concurrent hash table
   - **SplitOrderedList**: Ordered hash table variant for better cache locality

6. **ListSet** - Concurrent set implementations
   - **FineGrainedListSet**: Node-level locking for concurrency
   - **OptimisticFineGrainedListSet**: Optimistic locking to reduce contention

7. **ADT (Abstract Data Types)** - High-level concurrent collections
   - **ConcurrentMap**: Thread-safe key-value store
   - **ConcurrentSet**: Thread-safe set data structure

### Hello Server - Real-world Application

A practical concurrent web server demonstrating how to apply concurrent programming patterns to production systems:

- **TcpListener** - Cancellable TCP listener with atomic flags
  - Thread-safe cancellation mechanism
  - Graceful shutdown of connection acceptance

- **ThreadPool** - Work-stealing thread pool implementation
  - Fixed number of threads processing jobs concurrently
  - Automatic thread cleanup with `Drop`
  - Job synchronization using condition variables

- **Handler** - HTTP request processor with caching
  - Parses HTTP GET requests
  - Demonstrates expensive computation patterns
  - Integrates with cache for performance

- **Cache** - Fine-grained concurrent cache
  - Per-key locking to avoid blocking unrelated operations
  - Ensures expensive computations run only once per key
  - Uses HashMap Entry API for efficient concurrent updates

- **Statistics** - Server metrics tracking
  - Collects request statistics
  - Tracks cache hit/miss patterns

## Testing & Verification

Comprehensive testing using [LLVM Sanitizers](https://clang.llvm.org/docs/UndefinedBehaviorSanitizer.html):

- **AddressSanitizer**: Detects memory bugs (use-after-free, buffer overflows)
- **ThreadSanitizer**: Detects data races and synchronization issues

While Rust's type system guarantees memory safety and prevents data races in safe code, the correctness of `unsafe` Rust implementations relies on proper synchronization. Sanitizers are essential tools for verifying library correctness.

### Running Tests

```bash
# Run specific test module
cargo test --test <module_name>
# Example: cargo test --test hazard_pointer

# Run with AddressSanitizer
source scripts/grade-utils.sh
cargo_asan test --test <module_name>

# Run with ThreadSanitizer
cargo_tsan test --test <module_name>
```
