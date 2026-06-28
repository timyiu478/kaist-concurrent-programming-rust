* Memory (Consistency) model:
    * It is a contract between hardware and software.
    * It defines the allowed orderings of memory operations (reads and writes) when multiple threads run concurrently.
    * It answers the core question: When a thread changes a variable, when and in what order do other threads see that change?
* Sequential Consistency:
    * (1) Instructions are executed in program order
    * (2) Before executing the next instruction, the effect of the current instruction needs to be visble to all other threads
* Total Store Ordering/Store Buffer: One of the relaxed consistency model
    * Obey the single thread behaviour: read-my-own-write
    * All writes to the same location will be seen in the same order for all threads
    * How does it work? The writes are stored in the local store buffer first and then propagate the shared memory.
* A Happen-Before relationship: if a statement A happen-before B, the memory model guarantees that **all memory writes by A are visible to B**
    * When Thread A releases a lock, it doesn't just unlock the door; it forces a "memory flush" of everything it did while it held the lock. When Thread B subsequently acquires that same lock, it performs a "memory refresh," making all of Thread A's changes visible to it.

```c
// Shared variables
int data = 0;
bool ready = false;
mutex my_lock;

// --- THREAD A ---
data = 42;               // 1. Plain memory write
ready = true;            // 2. Plain memory write
lock_release(my_lock);   // 3. LOCK RELEASE (Synchronizes-with)

// --- THREAD B ---
lock_acquire(my_lock);   // 4. LOCK ACQUIRE
print(data);             // 5. Guaranteed to print 42!
print(ready);            // 6. Guaranteed to print true!
```


References:

* https://jamesbornholt.com/blog/memory-models/
* https://cseweb.ucsd.edu/classes/fa13/cse160-a/Lectures/Lec07.pdf
