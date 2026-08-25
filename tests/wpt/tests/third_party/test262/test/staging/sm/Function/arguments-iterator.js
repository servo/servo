// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
flags:
  - noStrict
description: |
  Implement arguments[@@iterator].
info: bugzilla.mozilla.org/show_bug.cgi?id=992617
esid: pending
---*/

// MappedArgumentsObject
let mapped = [
  function(a, b, c) {
    assert.sameValue(Symbol.iterator in arguments, true);
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function(a, b, c) {
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function(a, b, c) {
    arguments[Symbol.iterator] = 10;
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function(a, b, c) {
    Object.defineProperty(arguments, Symbol.iterator, {
      value: 10, writable: true, enumerable: true, configurable: true
    });
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function(a, b, c) {
    assert.sameValue(arguments[Symbol.iterator], Array.prototype[Symbol.iterator]);
  },
  function(a, b, c) {
    assert.sameValue(arguments[Symbol.iterator].name, "values");
  },
  function(a, b, c) {
    var desc = Object.getOwnPropertyDescriptor(arguments, Symbol.iterator);
    assert.sameValue("value" in desc, true);
    assert.sameValue(desc.value, Array.prototype[Symbol.iterator]);
    assert.sameValue(desc.writable, true);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.configurable, true);
  },
  function(a, b, c) {
    var iter = arguments[Symbol.iterator]();
    assert.deepEqual(iter.next(), { value: 10, done: false });
    assert.deepEqual(iter.next(), { value: 20, done: false });
    assert.deepEqual(iter.next(), { value: 30, done: false });
    assert.deepEqual(iter.next(), { value: undefined, done: true });
  },
  function(a, b, c) {
    assert.deepEqual([...arguments], [10, 20, 30]);
  },
  function(a, b, c) {
    b = 40;
    assert.deepEqual([...arguments], [10, 40, 30]);
  },
  function(a, b, c) {
    arguments.length = 4;
    assert.deepEqual([...arguments], [10, 20, 30, undefined]);
  },
  function(a, b, c) {
    arguments[5] = 50;
    assert.deepEqual([...arguments], [10, 20, 30]);
  },
  function(a, b, c) {
    arguments[Symbol.iterator] = function*() {
      yield 40;
      yield 50;
      yield 60;
    };
    assert.deepEqual([...arguments], [40, 50, 60]);
  },
];
for (let f of mapped) {
  f(10, 20, 30);
}

var g1 = $262.createRealm().global;
assert.sameValue(g1.eval(`
function f(a, b, c) {
  return arguments[Symbol.iterator].name;
}
f(1, 2, 3);
`), "values");

// UnmappedArgumentsObject
let unmapped = [
  function([a], b, c) {
    assert.sameValue(Symbol.iterator in arguments, true);
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function([a], b, c) {
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function([a], b, c) {
    arguments[Symbol.iterator] = 10;
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function([a], b, c) {
    Object.defineProperty(arguments, Symbol.iterator, {
      value: 10, writable: true, enumerable: true, configurable: true
    });
    delete arguments[Symbol.iterator];
    assert.sameValue(Symbol.iterator in arguments, false);
  },
  function([a], b, c) {
    assert.sameValue(arguments[Symbol.iterator], Array.prototype[Symbol.iterator]);
  },
  function([a], b, c) {
    assert.sameValue(arguments[Symbol.iterator].name, "values");
  },
  function([a], b, c) {
    var desc = Object.getOwnPropertyDescriptor(arguments, Symbol.iterator);
    assert.sameValue("value" in desc, true);
    assert.sameValue(desc.value, Array.prototype[Symbol.iterator]);
    assert.sameValue(desc.writable, true);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.configurable, true);
  },
  function([a], b, c) {
    var iter = arguments[Symbol.iterator]();
    assert.deepEqual(iter.next(), { value: [10], done: false });
    assert.deepEqual(iter.next(), { value: 20, done: false });
    assert.deepEqual(iter.next(), { value: 30, done: false });
    assert.deepEqual(iter.next(), { value: undefined, done: true });
  },
  function([a], b, c) {
    assert.deepEqual([...arguments], [[10], 20, 30]);
  },
  function([a], b, c) {
    b = 40;
    assert.deepEqual([...arguments], [[10], 20, 30]);
  },
  function([a], b, c) {
    arguments.length = 4;
    assert.deepEqual([...arguments], [[10], 20, 30, undefined]);
  },
  function([a], b, c) {
    arguments[5] = 50;
    assert.deepEqual([...arguments], [[10], 20, 30]);
  },
  function([a], b, c) {
    arguments[Symbol.iterator] = function*() {
      yield 40;
      yield 50;
      yield 60;
    };
    assert.deepEqual([...arguments], [40, 50, 60]);
  },
];
for (let f of unmapped) {
  f([10], 20, 30);
}

var g2 = $262.createRealm().global;
assert.sameValue(g2.eval(`
function f([a], b, c) {
  return arguments[Symbol.iterator].name;
}
f([1], 2, 3);
`), "values");
