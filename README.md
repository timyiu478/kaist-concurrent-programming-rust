# KAIST Concurrent Programming in Rust

This repository contains my implementation of concurrent data structures and libraries in **Rust**, from the [KAIST CS431](https://github.com/kaist-cp/cs431) course.

## Course Overview

**Understand the motivations and challenges of concurrent programming.** Learn design patterns and principles for reasoning about concurrency. Design, implement, and evaluate concurrent programs.

## Implementations

> [!IMPORTANT]
> The code here is offered as a learning aid to help you build intuition and see one possible way of solving the problem. Readers are strongly encouraged to engage actively with the material and develop their own independent implementations.

### [Hello Server](homework/src/hello_server)

A practical concurrent web server demonstrating how to apply concurrent programming patterns:

- **TcpListener** - Cancellable TCP listener with atomic flags
  - Thread-safe cancellation mechanism
  - Graceful shutdown of connection acceptance

- **ThreadPool** - Work-stealing thread pool implementation
  - Fixed number of threads processing jobs concurrently
  - Automatic thread cleanup with `Drop`

- **Cache** - Fine-grained concurrent cache
  - Per-key locking to avoid blocking unrelated operations
  - Ensures expensive computations run only once per key
  - Uses HashMap Entry API for efficient concurrent updates

- **Handler** - HTTP request processor with caching
  - Parses HTTP GET requests
  - Demonstrates expensive computation patterns
  - Integrates with cache for performance

- **Statistics** - Server metrics tracking
  - Collects request statistics
  - Tracks cache hit/miss patterns

### Core Concurrent Data Structures

1. **[Arc](homework/src/arc.rs)** - Atomic Reference Counting for shared ownership across threads
   - Enables safe shared access to heap data without garbage collection
   - Foundation for building other concurrent structures

2. **[TODO: BoC (Bag of Counts)](homework/src/boc.rs)** - A concurrency pattern using CownPtr
   - Demonstrates scalable concurrent counting
   - Lock-free approach to distributed counters

3. **[TODO: ElimStack](home/src/elim_stack)** - Elimination-based stack for concurrent access
   - Scales better than traditional locked stacks
   - Threads coordinate directly to eliminate push-pop pairs

4. **[TODO: LinkedList](home/src/linked_list.rs)** - Concurrent linked list implementation
   - Fine-grained locking and optimistic approaches
   - Handling concurrent insertions, deletions, and lookups

5. **[TODO: HashTable](home/src/hash_table)** - Concurrent hash table structures
   - **GrowableArray**: Dynamic resizing concurrent hash table
   - **SplitOrderedList**: Ordered hash table variant for better cache locality

6. **[TODO: ListSet](home/src/list_set)** - Concurrent set implementations
   - **FineGrainedListSet**: Node-level locking for concurrency
   - **OptimisticFineGrainedListSet**: Optimistic locking to reduce contention


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
