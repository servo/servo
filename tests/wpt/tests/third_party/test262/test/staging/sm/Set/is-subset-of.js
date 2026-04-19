// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.isSubsetOf, "function");
verifyProperty(Set.prototype.isSubsetOf, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.isSubsetOf, "name", {
  value: "isSubsetOf", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Empty set is a subset of the empty set.
assert.sameValue(emptySet.isSubsetOf(emptySet), true);
assert.sameValue(emptySet.isSubsetOf(emptySetLike), true);
assert.sameValue(emptySet.isSubsetOf(emptyMap), true);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Subset operation with an empty set.
  assert.sameValue(new Set(values).isSubsetOf(emptySet), values.length === 0);
  assert.sameValue(new Set(values).isSubsetOf(emptySetLike), values.length === 0);
  assert.sameValue(new Set(values).isSubsetOf(emptyMap), values.length === 0);
  assert.sameValue(emptySet.isSubsetOf(new Set(values)), true);
  assert.sameValue(emptySet.isSubsetOf(new SetLike(values)), true);
  assert.sameValue(emptySet.isSubsetOf(asMap(values)), true);

  // Two sets with the exact same values.
  assert.sameValue(new Set(values).isSubsetOf(new Set(values)), true);
  assert.sameValue(new Set(values).isSubsetOf(new SetLike(values)), true);
  assert.sameValue(new Set(values).isSubsetOf(asMap(values)), true);

  // Subset operation of the same set object.
  let set = new Set(values);
  assert.sameValue(set.isSubsetOf(set), true);
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

  assert.sameValue(emptySet.isSubsetOf(setLike), true);

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
      throw new Error("Unexpected call to |keys| method");
    },
  }, log);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSubsetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSubsetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSubsetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.isSubsetOf(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.isSubsetOf([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assert.sameValue(emptySet.isSubsetOf(myEmptySet), true);
  assert.sameValue(myEmptySet.isSubsetOf(myEmptySet), true);
  assert.sameValue(myEmptySet.isSubsetOf(emptySet), true);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([]);
  Object.setPrototypeOf(set, setProto);

  assert.sameValue(Set.prototype.isSubsetOf.call(set, emptySet), true);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.isSubsetOf.call(thisValue, emptySet));
}

// Doesn't call |has| when this-value has more elements.
{
  let set = new Set([1, 2, 3]);

  for (let size of [0, 1, 2]) {
    let setLike = {
      size,
      has() {
        throw new Error("Unexpected call to |has| method");
      },
      keys() {
        throw new Error("Unexpected call to |keys| method");
      },
    };

    assert.sameValue(set.isSubsetOf(setLike), false);
  }
}

// Test when this-value is modified during iteration.
{
  let set = new Set([1, 2, 3]);

  let seen = new Set();

  let setLike = {
    size: 100,
    has(v) {
      assert.sameValue(arguments.length, 1);
      assert.sameValue(this, setLike);
      assert.sameValue(set.has(v), true);

      assert.sameValue(seen.has(v), false);
      seen.add(v);

      // Delete the current element.
      set.delete(v);

      return true;
    },
    keys() {
      throw new Error("Unexpected call to |keys| method");
    },
  };

  assert.sameValue(set.isSubsetOf(setLike), true);
  assertSetContainsExactOrderedItems(set, []);
  assertSetContainsExactOrderedItems(seen, [1, 2, 3]);
}
{
  let set = new Set([1]);

  let seen = new Set();
  let newKeys = [2, 3, 4, 5];

  let setLike = {
    size: 100,
    has(v) {
      assert.sameValue(arguments.length, 1);
      assert.sameValue(this, setLike);
      assert.sameValue(set.has(v), true);

      assert.sameValue(seen.has(v), false);
      seen.add(v);

      // Delete the current element.
      set.delete(v);

      // Add new elements.
      if (newKeys.length) {
        set.add(newKeys.shift());
      }

      return true;
    },
    keys() {
      throw new Error("Unexpected call to |keys| method");
    },
  };

  assert.sameValue(set.isSubsetOf(setLike), true);
  assertSetContainsExactOrderedItems(set, []);
  assertSetContainsExactOrderedItems(seen, [1, 2, 3, 4, 5]);
  assert.sameValue(newKeys.length, 0);
}
{
  let set = new Set([1, 2, 3]);

  let seen = new Set();
  let deleted = false;

  let setLike = {
    size: 100,
    has(v) {
      assert.sameValue(arguments.length, 1);
      assert.sameValue(this, setLike);
      assert.sameValue(set.has(v), true);

      assert.sameValue(seen.has(v), false);
      seen.add(v);

      if (!deleted) {
        assert.sameValue(v, 1);
        set.delete(2);
        deleted = true;
      }

      return true;
    },
    keys() {
      throw new Error("Unexpected call to |keys| method");
    },
  };

  assert.sameValue(set.isSubsetOf(setLike), true);
  assertSetContainsExactOrderedItems(set, [1, 3]);
  assertSetContainsExactOrderedItems(seen, [1, 3]);
  assert.sameValue(deleted, true);
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
  assert.sameValue(set.isSubsetOf(other), true);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

