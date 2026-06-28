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

References:

* https://jamesbornholt.com/blog/memory-models/
