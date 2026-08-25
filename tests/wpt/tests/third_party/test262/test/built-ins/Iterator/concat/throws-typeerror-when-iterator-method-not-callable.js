// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the iterator method is not callable.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    a. If item is not an Object, throw a TypeError exception.
    b. Let method be ? GetMethod(item, %Symbol.iterator%).
    c. If method is undefined, throw a TypeError exception.
    ...
features: [iterator-sequencing]
---*/

assert.throws(TypeError, function() {
  Iterator.concat({});
}, "iterable has no iterator method");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: undefined});
}, "iterator method is undefined");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: null});
}, "iterator method is null");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: true});
}, "iterator method is boolean");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: 123});
}, "iterator method is number");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: 123n});
}, "iterator method is bigint");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: "abc"});
}, "iterator method is string");

assert.throws(TypeError, function() {
  Iterator.concat({[Symbol.iterator]: Symbol()});
}, "iterator method is symbol");
