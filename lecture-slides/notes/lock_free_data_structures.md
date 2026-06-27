lock freedom: at least one of the thread make progress

* challenges: thread pause (due to I/O, crash), deadlock, livelock
* key idea of the solution: single-instruction commit point
    * no lock -> no deadlock, livelock
    * single-instruction -> no pause in the middle of the execution, either abort or commit
    * the commit point is the single contention point -> scalability

Treiber's Stack: push/pop synchronize with CAS on the top location

* data structure: a linked list
    * top -> 42 -> 37 -> 666 -> null
* pop: 
    * Read top -> 42
    * Read 42 -> 37
    * CAS top to 37
* push(100):
    * 100 -> 42
    * CAS top to 100

* why defer destory head in pop()?
    * 
