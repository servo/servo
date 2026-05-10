// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the iterable is not an object.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    a. If item is not an Object, throw a TypeError exception.
    ...
features: [iterator-sequencing]
---*/

assert.throws(TypeError, function() {
  Iterator.concat(undefined);
}, "iterable is undefined");

assert.throws(TypeError, function() {
  Iterator.concat(null);
}, "iterable is null");

assert.throws(TypeError, function() {
  Iterator.concat(true);
}, "iterable is boolean");

assert.throws(TypeError, function() {
  Iterator.concat(123);
}, "iterable is number");

assert.throws(TypeError, function() {
  Iterator.concat(123n);
}, "iterable is bigint");

assert.throws(TypeError, function() {
  Iterator.concat("test");
}, "iterable is string");

assert.throws(TypeError, function() {
  Iterator.concat(Symbol());
}, "iterable is symbol");
