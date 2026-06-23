* Release Semantics (The "No Upward Movement" Barrier)
    * It means the above instructions cannot leak down past the release instruction.

```
Instruction A (Write Data)
Instruction B
=================================== [ RELEASE FENCE ] (Instructions cannot move DOWN past this line)
Instruction C (Unlock)
```

* Acquire Semantics (The "No Downward Movement" Barrier)
    * It means the below instructions cannot leak up past the acquire instruction.

```
Instruction X (Lock / Acquire)
=================================== [ ACQUIRE FENCE ] (Instructions cannot move UP past this line)
Instruction Y (Read Data)
Instruction Z
```

* source of nondeterminisms:
    * multiple threads: threads interleaving
    * single thread: instructions reordering

* Reordering: unless accessing the same location, any two load/store/read-modify-write instructions can be reordered

* SC(Sequential Consistency) fenece: forbinning any reording across itself
    * a two way barrier

```
Instruction X
Instruction Y
==================================
SC fence
==================================
Instruction Z
```

