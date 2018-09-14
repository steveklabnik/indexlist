# `indexlist` - A doubly linked list, backed by a vector.

[![Build Status](https://travis-ci.org/steveklabnik/indexlist.svg?branch=master)](https://travis-ci.org/steveklabnik/indexlist) [![Build status](https://ci.appveyor.com/api/projects/status/baop9rw1tnd193or/branch/master?svg=true)](https://ci.appveyor.com/project/steveklabnik/indexlist/branch/master)

This crate provides a struct, `IndexList<T>`, which is a doubly-linked
list. However, unlike a traditional linked list, which heap allocates
each of its nodes individually, all nodes are stored in a vector. Rather
than provide pointers to nodes, an `Index` struct can be used to access
a particular elemnt in the middle of the list.

# Safety

This crate uses `#![deny(unsafe_code)]` to ensure everything is implemented
in 100% Safe Rust.

# Generational indexes

`Index` uses a generations scheme, so that if you hold an `Index` to a node,
and it's removed, and a new node is allocated in its place, you do not access
the new node.

# Performance

In general, performance is quite good. Benchmarks against the standard library's
`LinkedList<T>` are provided. But some other details:

* The list keeps track of its head and tail for efficient insertion.
* The underlying vector only grows, never shrinks. When a node is removed, its
  entry is marked as free for future insertions.
* Free entries are themselves kept as a singly-linked list, meaning that they
  can be re-used efficiently.

# Missing features

Right now, I've only implemented a minimal number of features; there's `iter`
but no `into_iter` and `iter_mut`. This is on the to-do list. PRs welcome!

