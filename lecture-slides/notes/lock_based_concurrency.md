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
