// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the iterator is not an object.
info: |
  Iterator.concat ( ...items )

  ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      i. Let iter be ? Call(iterable.[[OpenMethod]], iterable.[[Iterable]]).
      ii. If iter is not an Object, throw a TypeError exception.
      ...
features: [iterator-sequencing]
---*/

function MakeIterable(iterator) {
  return {
    [Symbol.iterator]() {
      return iterator;
    }
  };
}

var iterator;

iterator = Iterator.concat(MakeIterable(undefined));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is undefined");

iterator = Iterator.concat(MakeIterable(null));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is null");

iterator = Iterator.concat(MakeIterable(true));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is boolean");

iterator = Iterator.concat(MakeIterable(123));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is number");

iterator = Iterator.concat(MakeIterable(123n));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is bigint");

iterator = Iterator.concat(MakeIterable("abc"));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is string");

iterator = Iterator.concat(MakeIterable(Symbol()));
assert.throws(TypeError, function() { iterator.next(); }, "iterator is symbol");
