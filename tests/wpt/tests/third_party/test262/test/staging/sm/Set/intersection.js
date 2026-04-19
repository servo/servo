// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.intersection, "function");
verifyProperty(Set.prototype.intersection, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.intersection, "name", {
  value: "intersection", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Intersection of two empty sets is an empty set.
assertSetContainsExactOrderedItems(emptySet.intersection(emptySet), []);
assertSetContainsExactOrderedItems(emptySet.intersection(emptySetLike), []);
assertSetContainsExactOrderedItems(emptySet.intersection(emptyMap), []);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Intersection with an empty set.
  assertSetContainsExactOrderedItems(new Set(values).intersection(emptySet), []);
  assertSetContainsExactOrderedItems(new Set(values).intersection(emptySetLike), []);
  assertSetContainsExactOrderedItems(new Set(values).intersection(emptyMap), []);
  assertSetContainsExactOrderedItems(emptySet.intersection(new Set(values)), []);
  assertSetContainsExactOrderedItems(emptySet.intersection(new SetLike(values)), []);
  assertSetContainsExactOrderedItems(emptySet.intersection(asMap(values)), []);

  // Two sets with the exact same values.
  assertSetContainsExactOrderedItems(new Set(values).intersection(new Set(values)), values);
  assertSetContainsExactOrderedItems(new Set(values).intersection(new SetLike(values)), values);
  assertSetContainsExactOrderedItems(new Set(values).intersection(asMap(values)), values);

  // Intersection of the same set object.
  let set = new Set(values);
  assertSetContainsExactOrderedItems(set.intersection(set), values);
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

  log.length = 0;
  assertSetContainsExactOrderedItems(emptySet.intersection(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Input has more elements than the this-value.
  sizeValue = 1;

  log.length = 0;
  assertSetContainsExactOrderedItems(emptySet.intersection(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Input has fewer elements than the this-value.
  sizeValue = 0;

  log.length = 0;
  assertSetContainsExactOrderedItems(new Set([1]).intersection(setLike), []);

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
  assertSetContainsExactOrderedItems(emptySet.intersection(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  log.length = 0;
  assert.throws(TypeError, () => new Set([1]).intersection(setLike));

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
  assertSetContainsExactOrderedItems(emptySet.intersection(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  log.length = 0;
  assert.throws(TypeError, () => new Set([1]).intersection(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.intersection(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.intersection(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.intersection(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.intersection(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.intersection([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assertSetContainsExactOrderedItems(emptySet.intersection(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.intersection(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.intersection(emptySet), []);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([1, 2, 3]);
  Object.setPrototypeOf(set, setProto);

  assertSetContainsExactOrderedItems(Set.prototype.intersection.call(set, emptySet), []);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.intersection.call(thisValue, emptySet));
}

// Doesn't return the original Set object.
{
  let set = new Set([1]);
  assert.sameValue(set.intersection(emptySet) !== set, true);
  assert.sameValue(set.intersection(new Set([2])) !== set, true);
}

// Test insertion order
{
  let set = new Set([1, 2, 3]);

  // Case 1: Input is empty.
  assertSetContainsExactOrderedItems(set.intersection(new Set([])), []);

  // Case 2: Input has fewer elements.
  assertSetContainsExactOrderedItems(set.intersection(new Set([1, 2])), [1, 2]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([2, 1])), [2, 1]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([11, 2])), [2]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([2, 11])), [2]);

  // Case 3: Input has same number of elements.
  assertSetContainsExactOrderedItems(set.intersection(new Set([1, 2, 3])), [1, 2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([2, 3, 1])), [1, 2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([3, 2, 1])), [1, 2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([11, 2, 3])), [2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([2, 3, 11])), [2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([3, 2, 11])), [2, 3]);

  // Case 4: Input has more elements.
  assertSetContainsExactOrderedItems(set.intersection(new Set([2, 3, 4, 5])), [2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([4, 5, 2, 3])), [2, 3]);
  assertSetContainsExactOrderedItems(set.intersection(new Set([5, 4, 3, 2])), [2, 3]);
}

// Calls |has| when the this-value has fewer or the same number of keys.
{
  const keys = [1, 2, 3];

  for (let size of [keys.length, 100, Infinity]) {
    let i = 0;

    let setLike = {
      size,
      has(v) {
        assert.sameValue(this, setLike);
        assert.sameValue(arguments.length, 1);
        assert.sameValue(i < keys.length, true);
        assert.sameValue(v, keys[i++]);
        return true;
      },
      keys() {
        throw new Error("Unexpected call to |keys| method");
      },
    };

    assertSetContainsExactOrderedItems(new Set(keys).intersection(setLike), keys);
  }
}

// Calls |keys| when the this-value has more keys.
{
  const keys = [1, 2, 3];

  for (let size of [0, 1, 2]) {
    let i = 0;

    let setLike = {
      size,
      has() {
        throw new Error("Unexpected call to |keys| method");
      },
      keys() {
        assert.sameValue(this, setLike);
        assert.sameValue(arguments.length, 0);

        let iterator = {
          next() {
            assert.sameValue(this, iterator);
            assert.sameValue(arguments.length, 0);
            if (i < keys.length) {
              return {
                done: false,
                value: keys[i++],
              };
            }
            return {
              done: true,
              get value() {
                throw new Error("Unexpected call to |value| getter");
              },
            };
          }
        };

        return iterator;
      },
    };

    assertSetContainsExactOrderedItems(new Set(keys).intersection(setLike), keys);
  }
}

// Test result set order when the this-value was modified.
{
  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    *keys() {
      // Yield the same keys as in |set|.
      yield* set.keys();

      // Remove all existing items.
      set.clear();

      // Re-add keys 2 and 3, but in reversed order.
      set.add(3);
      set.add(2);

      // Additionally add 99.
      set.add(99);
    },
  };

  let set = new Set([1, 2, 3, 4]);

  assertSetContainsExactOrderedItems(set.intersection(setLike), [1, 2, 3, 4]);
  assertSetContainsExactOrderedItems(set, [3, 2, 99]);
}
{
  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    *keys() {
      // Yield the same keys as in |set|.
      yield* set.keys();

      // Remove only keys 2 and 3.
      set.delete(2);
      set.delete(3);

      // Re-add keys 2 and 3, but in reversed order.
      set.add(3);
      set.add(2);
    },
  };

  let set = new Set([1, 2, 3, 4]);

  assertSetContainsExactOrderedItems(set.intersection(setLike), [1, 2, 3, 4]);
  assertSetContainsExactOrderedItems(set, [1, 4, 3, 2]);
}

// Test the same item can't be added multiple times.
{
  let seen = [];

  let setLike = {
    size: 100,
    has(v) {
      // Remove and then re-add 2.
      if (v === 2 && !seen.includes(v)) {
        set.delete(v);
        set.add(v);
      }

      // Remember all visited keys.
      seen.push(v);

      return true;
    },
    keys() {
      throw new Error("Unexpected call to |keys| method");
    },
  };

  let set = new Set([1, 2, 3]);

  assertSetContainsExactOrderedItems(set.intersection(setLike), [1, 2, 3]);
  assert.compareArray(seen, [1, 2, 3, 2]);
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
  assertSetContainsExactOrderedItems(set.intersection(other), [1, 2, 3]);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

