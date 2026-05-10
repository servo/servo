// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.isSupersetOf, "function");
verifyProperty(Set.prototype.isSupersetOf, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.isSupersetOf, "name", {
  value: "isSupersetOf", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Empty set is a superset of the empty set.
assert.sameValue(emptySet.isSupersetOf(emptySet), true);
assert.sameValue(emptySet.isSupersetOf(emptySetLike), true);
assert.sameValue(emptySet.isSupersetOf(emptyMap), true);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Superset operation with an empty set.
  assert.sameValue(new Set(values).isSupersetOf(emptySet), true);
  assert.sameValue(new Set(values).isSupersetOf(emptySetLike), true);
  assert.sameValue(new Set(values).isSupersetOf(emptyMap), true);
  assert.sameValue(emptySet.isSupersetOf(new Set(values)), values.length === 0);
  assert.sameValue(emptySet.isSupersetOf(new SetLike(values)), values.length === 0);
  assert.sameValue(emptySet.isSupersetOf(asMap(values)), values.length === 0);

  // Two sets with the exact same values.
  assert.sameValue(new Set(values).isSupersetOf(new Set(values)), true);
  assert.sameValue(new Set(values).isSupersetOf(new SetLike(values)), true);
  assert.sameValue(new Set(values).isSupersetOf(asMap(values)), true);

  // Superset operation of the same set object.
  let set = new Set(values);
  assert.sameValue(set.isSupersetOf(set), true);
}

// Check property accesses are in the correct order.
{
  let log = [];

  let sizeValue = 0;

  let {proxy: keysIter} = LoggingProxy({
    next() {
      log.push("next()");
      return {done: true};
    }
  }, log);

  let {proxy: setLike} = LoggingProxy({
    size: {
      valueOf() {
        log.push("valueOf()");
        return sizeValue;
      }
    },
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    keys() {
      log.push("keys()");
      return keysIter;
    },
  }, log);

  assert.sameValue(emptySet.isSupersetOf(setLike), true);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
    "keys()",
    "[[get]]", "next",
    "next()",
  ]);

  // |keys| isn't called when the this-value has fewer elements.
  sizeValue = 1;

  log.length = 0;
  assert.sameValue(emptySet.isSupersetOf(setLike), false);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);
}

// Check input validation.
{
  let log = [];

  const nonCallable = {};
  let sizeValue = 0;

  let {proxy: keysIter} = LoggingProxy({
    next: nonCallable,
  }, log);

  let {proxy: setLike, obj: setLikeObj} = LoggingProxy({
    size: {
      valueOf() {
        log.push("valueOf()");
        return sizeValue;
      }
    },
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    keys() {
      log.push("keys()");
      return keysIter;
    },
  }, log);

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
    "keys()",
    "[[get]]", "next",
  ]);

  // Change |keys| to return a non-object value.
  setLikeObj.keys = () => 123;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSupersetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.isSupersetOf([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assert.sameValue(emptySet.isSupersetOf(myEmptySet), true);
  assert.sameValue(myEmptySet.isSupersetOf(myEmptySet), true);
  assert.sameValue(myEmptySet.isSupersetOf(emptySet), true);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([]);
  Object.setPrototypeOf(set, setProto);

  assert.sameValue(Set.prototype.isSupersetOf.call(set, emptySet), true);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.isSupersetOf.call(thisValue, emptySet));
}

// Doesn't call |has| nor |keys| when this-value has fewer elements.
{
  let set = new Set([1, 2, 3]);

  for (let size of [100, Infinity]) {
    let setLike = {
      size,
      has() {
        throw new Error("Unexpected call to |has| method");
      },
      keys() {
        throw new Error("Unexpected call to |keys| method");
      },
    };

    assert.sameValue(set.isSupersetOf(setLike), false);
  }
}

// Test when this-value is modified during iteration.
{
  let set = new Set([]);

  let keys = [1, 2, 3];

  let keysIter = {
    next() {
      if (keys.length) {
        let value = keys.shift();
        return {
          done: false,
          get value() {
            assert.sameValue(set.has(value), false);
            set.add(value);
            return value;
          }
        };
      }
      return {
        done: true,
        get value() {
          throw new Error("Unexpected call to |value| getter");
        }
      };
    }
  };

  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    keys() {
      return keysIter;
    },
  };

  assert.sameValue(set.isSupersetOf(setLike), true);
  assertSetContainsExactOrderedItems(set, [1, 2, 3]);
}

// IteratorClose is called for early returns.
{
  let log = [];

  let keysIter = {
    next() {
      log.push("next");
      return {done: false, value: 1};
    },
    return() {
      log.push("return");
      return {
        get value() { throw new Error("Unexpected call to |value| getter"); },
        get done() { throw new Error("Unexpected call to |done| getter"); },
      };
    }
  };

  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    keys() {
      return keysIter;
    },
  };

  assert.sameValue(new Set([2, 3, 4]).isSupersetOf(setLike), false);

  assert.compareArray(log, ["next", "return"]);
}

// IteratorClose isn't called for non-early returns.
{
  let setLike = new SetLike([1, 2, 3]);

  assert.sameValue(new Set([1, 2, 3]).isSupersetOf(setLike), true);
}

// Tests which modify any built-ins should appear last, because modifications may disable
// optimised code paths.

// Doesn't call the built-in |Set.prototype.{has, keys, size}| functions.
const SetPrototypeHas = Object.getOwnPropertyDescriptor(Set.prototype, "has");
const SetPrototypeKeys = Object.getOwnPropertyDescriptor(Set.prototype, "keys");
const SetPrototypeSize = Object.getOwnPropertyDescriptor(Set.prototype, "size");

delete Set.prototype.has;
delete Set.prototype.keys;
delete Set.prototype.size;

try {
  let set = new Set([1, 2, 3]);
  let other = new SetLike([1, 2, 3]);
  assert.sameValue(set.isSupersetOf(other), true);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

