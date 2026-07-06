* Cown: concurrent owned object
    * state: available or acquired by behavior

* Behavior: list of cowns + closure
    * when can be run: all cowns are available and all other behaviours which happen before it have been run
    * behavior can acquire multiple cowns at once
        * e.g. list 5
    * for hehaviors requiring non-overlapping cowns, the order of execution does not matter
    * The actual execution of the behavior's thunk is entirely **reactive**, triggered downstream by the resource dependencies themselves as they become free.
