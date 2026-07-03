---
title: "Split-Ordered Lists: Lock-Free Extensible Hash Tables"
---

# Key Ideas

* All elements are stored in a single lock-free linked list.
* The elements are ordered in a way that they can be splitted recursively using **binary reversal**. 
* To distinguish dummy node and regular node, set the MSB bit to one for all regular key.
* The resizing algo moves buckets among items.
* They bucket of item k is k % bucket size = the LSBs of k.

# Motivations

* Resizing is needed to make sure each bucket has **constant** size of elements.
* Fine-grained locking hash table requires **stop the world** when resizing.
* No lock-free hash table was extensible at that time.

# Questions

## Q. What makes lock-free extensible hashing hard to achieve?

When resizing, some items need to move from one bucket to another bucket.
But a single CAS operation can't move one item from one bucket to another bucket. 

## Q. What is the purpose of dummy node in the linked list?

It avoids the regular node will have two pointers point to it(one is its previous node, another one is the bucket).
If a regular node have two pointers point to it, the deletion operation is hard to implement(can't simply use one CAS).

## Q. Why recursive initialization?

Because to initial a bucket, we need to know where to insert the dummy node.
To know where to insert the dummy node, we need to the location of the parent of the dummy node.
