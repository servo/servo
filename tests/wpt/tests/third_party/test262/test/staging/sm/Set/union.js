// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.union, "function");
verifyProperty(Set.prototype.union, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.union, "name", {
  value: "union", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Union of two empty sets is an empty set.
assertSetContainsExactOrderedItems(emptySet.union(emptySet), []);
assertSetContainsExactOrderedItems(emptySet.union(emptySetLike), []);
assertSetContainsExactOrderedItems(emptySet.union(emptyMap), []);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Union with an empty set.
  assertSetContainsExactOrderedItems(new Set(values).union(emptySet), values);
  assertSetContainsExactOrderedItems(new Set(values).union(emptySetLike), values);
  assertSetContainsExactOrderedItems(new Set(values).union(emptyMap), values);
  assertSetContainsExactOrderedItems(emptySet.union(new Set(values)), values);
  assertSetContainsExactOrderedItems(emptySet.union(new SetLike(values)), values);
  assertSetContainsExactOrderedItems(emptySet.union(asMap(values)), values);

  // Two sets with the exact same values.
  assertSetContainsExactOrderedItems(new Set(values).union(new Set(values)), values);
  assertSetContainsExactOrderedItems(new Set(values).union(new SetLike(values)), values);
  assertSetContainsExactOrderedItems(new Set(values).union(asMap(values)), values);

  // Union of the same set object.
  let set = new Set(values);
  assertSetContainsExactOrderedItems(set.union(set), values);
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

  assertSetContainsExactOrderedItems(emptySet.union(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
    "keys()",
    "[[get]]", "next",
    "next()",
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
  assert.throws(TypeError, () => emptySet.union(setLike));

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
  assert.throws(TypeError, () => emptySet.union(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.union(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.union(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.union(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.union(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.union([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assertSetContainsExactOrderedItems(emptySet.union(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.union(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.union(emptySet), []);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([1, 2, 3]);
  Object.setPrototypeOf(set, setProto);

  assertSetContainsExactOrderedItems(Set.prototype.union.call(set, emptySet), [1, 2, 3]);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.union.call(thisValue, emptySet));
}

// Doesn't return the original Set object.
{
  let set = new Set([1]);
  assert.sameValue(set.union(emptySet) !== set, true);
  assert.sameValue(set.union(new Set([2])) !== set, true);
}

// Test insertion order
{
  let set = new Set([1, 2]);

  // Case 1: Input is empty.
  assertSetContainsExactOrderedItems(set.union(new Set([])), [1, 2]);

  // Case 2: Input has fewer elements.
  assertSetContainsExactOrderedItems(set.union(new Set([3])), [1, 2, 3]);

  // Case 3: Input has same number of elements.
  assertSetContainsExactOrderedItems(set.union(new Set([3, 4])), [1, 2, 3, 4]);

  // Case 4: Input has more elements.
  assertSetContainsExactOrderedItems(set.union(new Set([3, 4, 5])), [1, 2, 3, 4, 5]);
}

// Test that the input set is copied after accessing the |next| property of the keys iterator.
{
  let set = new Set([1, 2, 3]);

  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    keys() {
      return {
        get next() {
          // Clear the set when getting the |next| method.
          set.clear();

          // And then add a single new key.
          set.add(4);

          return function() {
            return {done: true};
          };
        }
      };
    },
  };

  // The result should consist of the single, newly added key.
  assertSetContainsExactOrderedItems(set.union(setLike), [4]);
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
  assertSetContainsExactOrderedItems(set.union(other), [1, 2, 3]);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

