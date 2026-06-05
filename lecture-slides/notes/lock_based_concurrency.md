Lock-based concurrency low-level API:

* lock, unlock, try_lock
    * cons: it is not easy to acquire/release the RIGHT lock at the RIGHT time
        * fix: design a higher level of API 
            * AUTO acquire and release the lock
            * Lock and resource are explicit related

Lock-based concurrency high-level API:

* Lockguard: 
    * RAII: lock release when the lock guard is destructed (when the lock guard object goes out of scope)
    * Locked-data: a pair of lock and data
        * pro: create a relationship of lock and data that which data is protect which lock
    * How does rust prevent the data pointer outlive the data guard?

Interior Mutability:

* encapsulate unsafe operations into safe API
* borrow checker executes in runtime

Send vs Sync trait:

* Send
    * Safe to move ownership to another thread
    * Examples: "String, Vec, Box"
    * Non-Examples: Rc (reference counting is not atomic)
* Sync
    * Safe to share between threads via immutable references (&T)
    * Examples: "Mutex, RwLock, AtomicBool"
    * Non-Examples: "RefCell, Cell (interior mutability is not thread-safe)"

Safe API example:

* spawn
    * it requires Send + 'static constraints on the closure
    * 'static: rust guarantee new thread won't outlive the data its using => no dangling pointer
        * forces you to use the move keyword to completely give up ownership of local variables to the thread
    * Send
        * variables that are not safe to send to other threads will be rejected by rust compiler => unable to use non thread-safe data structures

Q. Why Arc<Mutex<T>>? 

* The separation of concerns
    * Arc: handles the data sharding between threads and data lifetime (Mutex stays alive on the heap as long as at least one thread is still holding a clone of the Arc.)
    * Mutex: handles modifiability of the data

Parking lot library: Conditional Variable

* example: https://doc.rust-lang.org/std/sync/struct.Condvar.html
    * let (lock, cvar) = &*pair2; // get the shared reference of the underlying tuple

Aliasing XOR Mutability:

* the following code is wrong because immutable reference s lifetime intersects with mutable reference variable
* the overlapping lifetimes causing the violation of Aliasing XOR Mutability

```rust
let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();
let s = &started;
while !*started {
    started = cvar.wait(started).unwrap();
}
println!("{}", s);
```

Lock API:

* Rust does not guarantee that unlock is only be called by the owner. Its the responsibility of the API user.
* User should pass the token he obtain from the lock() when he calls unlock() as a proof of ownership

