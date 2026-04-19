// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.symmetricDifference, "function");
verifyProperty(Set.prototype.symmetricDifference, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.symmetricDifference, "name", {
  value: "symmetricDifference", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Symmetric difference of two empty sets is an empty set.
assertSetContainsExactOrderedItems(emptySet.symmetricDifference(emptySet), []);
assertSetContainsExactOrderedItems(emptySet.symmetricDifference(emptySetLike), []);
assertSetContainsExactOrderedItems(emptySet.symmetricDifference(emptyMap), []);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Symmetric difference with an empty set.
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(emptySet), values);
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(emptySetLike), values);
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(emptyMap), values);
  assertSetContainsExactOrderedItems(emptySet.symmetricDifference(new Set(values)), values);
  assertSetContainsExactOrderedItems(emptySet.symmetricDifference(new SetLike(values)), values);
  assertSetContainsExactOrderedItems(emptySet.symmetricDifference(asMap(values)), values);

  // Two sets with the exact same values.
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(new Set(values)), []);
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(new SetLike(values)), []);
  assertSetContainsExactOrderedItems(new Set(values).symmetricDifference(asMap(values)), []);

  // Symmetric difference of the same set object.
  let set = new Set(values);
  assertSetContainsExactOrderedItems(set.symmetricDifference(set), []);
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

  assertSetContainsExactOrderedItems(emptySet.symmetricDifference(setLike), []);

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
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

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
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.symmetricDifference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.symmetricDifference([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assertSetContainsExactOrderedItems(emptySet.symmetricDifference(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.symmetricDifference(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.symmetricDifference(emptySet), []);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([1, 2, 3]);
  Object.setPrototypeOf(set, setProto);

  assertSetContainsExactOrderedItems(Set.prototype.symmetricDifference.call(set, emptySet), [1, 2, 3]);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.symmetricDifference.call(thisValue, emptySet));
}

// Doesn't return the original Set object.
{
  let set = new Set([1]);
  assert.sameValue(set.symmetricDifference(emptySet) !== set, true);
  assert.sameValue(set.symmetricDifference(new Set([2])) !== set, true);
}

// Test insertion order
{
  let set = new Set([1, 2]);

  // Case 1: Input is empty.
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([])), [1, 2]);

  // Case 2: Input has fewer elements.
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([3])), [1, 2, 3]);
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([2])), [1]);

  // Case 3: Input has same number of elements.
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([3, 4])), [1, 2, 3, 4]);
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([2, 3])), [1, 3]);

  // Case 4: Input has more elements.
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([3, 4, 5])), [1, 2, 3, 4, 5]);
  assertSetContainsExactOrderedItems(set.symmetricDifference(new Set([2, 4, 5])), [1, 4, 5]);
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
  assertSetContainsExactOrderedItems(set.symmetricDifference(setLike), [4]);
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
  assertSetContainsExactOrderedItems(set.symmetricDifference(other), []);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

