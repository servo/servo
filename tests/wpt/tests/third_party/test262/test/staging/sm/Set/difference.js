// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Set-shell.js, compareArray.js, propertyHelper.js]
description: |
  pending
esid: pending
---*/

assert.sameValue(typeof Set.prototype.difference, "function");
verifyProperty(Set.prototype.difference, "length", {
  value: 1, writable: false, enumerable: false, configurable: true,
});
verifyProperty(Set.prototype.difference, "name", {
  value: "difference", writable: false, enumerable: false, configurable: true,
});

const emptySet = new Set();
const emptySetLike = new SetLike();
const emptyMap = new Map();

function asMap(values) {
  return new Map(values.map(v => [v, v]));
}

// Difference of two empty sets is an empty set.
assertSetContainsExactOrderedItems(emptySet.difference(emptySet), []);
assertSetContainsExactOrderedItems(emptySet.difference(emptySetLike), []);
assertSetContainsExactOrderedItems(emptySet.difference(emptyMap), []);

// Test native Set, Set-like, and Map objects.
for (let values of [
  [], [1], [1, 2], [1, true, null, {}],
]) {
  // Difference with an empty set.
  assertSetContainsExactOrderedItems(new Set(values).difference(emptySet), values);
  assertSetContainsExactOrderedItems(new Set(values).difference(emptySetLike), values);
  assertSetContainsExactOrderedItems(new Set(values).difference(emptyMap), values);
  assertSetContainsExactOrderedItems(emptySet.difference(new Set(values)), []);
  assertSetContainsExactOrderedItems(emptySet.difference(new SetLike(values)), []);
  assertSetContainsExactOrderedItems(emptySet.difference(asMap(values)), []);

  // Two sets with the exact same values.
  assertSetContainsExactOrderedItems(new Set(values).difference(new Set(values)), []);
  assertSetContainsExactOrderedItems(new Set(values).difference(new SetLike(values)), []);
  assertSetContainsExactOrderedItems(new Set(values).difference(asMap(values)), []);

  // Difference of the same set object.
  let set = new Set(values);
  assertSetContainsExactOrderedItems(set.difference(set), []);
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
  assertSetContainsExactOrderedItems(emptySet.difference(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Input has more elements than the this-value.
  sizeValue = 1;

  log.length = 0;
  assertSetContainsExactOrderedItems(emptySet.difference(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Input has fewer elements than the this-value.
  sizeValue = 0;

  log.length = 0;
  assertSetContainsExactOrderedItems(new Set([1]).difference(setLike), [1]);

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
  assertSetContainsExactOrderedItems(emptySet.difference(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  log.length = 0;
  assert.throws(TypeError, () => new Set([1]).difference(setLike));

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
  assertSetContainsExactOrderedItems(emptySet.difference(setLike), []);

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  log.length = 0;
  assert.throws(TypeError, () => new Set([1]).difference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |keys| to a non-callable value.
  setLikeObj.keys = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.difference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
    "[[get]]", "keys",
  ]);

  // Change |has| to a non-callable value.
  setLikeObj.has = nonCallable;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.difference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
    "[[get]]", "has",
  ]);

  // Change |size| to NaN.
  sizeValue = NaN;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.difference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);

  // Change |size| to undefined.
  sizeValue = undefined;

  log.length = 0;
  assert.throws(TypeError, () => emptySet.difference(setLike));

  assert.compareArray(log, [
    "[[get]]", "size",
    "valueOf()",
  ]);
}

// Doesn't accept Array as an input.
assert.throws(TypeError, () => emptySet.difference([]));

// Works with Set subclasses.
{
  class MySet extends Set {}

  let myEmptySet = new MySet;

  assertSetContainsExactOrderedItems(emptySet.difference(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.difference(myEmptySet), []);
  assertSetContainsExactOrderedItems(myEmptySet.difference(emptySet), []);
}

// Doesn't access any properties on the this-value.
{
  let log = [];

  let {proxy: setProto} = LoggingProxy(Set.prototype, log);

  let set = new Set([1, 2, 3]);
  Object.setPrototypeOf(set, setProto);

  assertSetContainsExactOrderedItems(Set.prototype.difference.call(set, emptySet), [1, 2, 3]);

  assert.compareArray(log, []);
}

// Throws a TypeError when the this-value isn't a Set object.
for (let thisValue of [
  null, undefined, true, "", {}, new Map, new Proxy(new Set, {}),
]) {
  assert.throws(TypeError, () => Set.prototype.difference.call(thisValue, emptySet));
}

// Doesn't return the original Set object.
{
  let set = new Set([1]);
  assert.sameValue(set.difference(emptySet) !== set, true);
  assert.sameValue(set.difference(new Set([2])) !== set, true);
}

// Test insertion order
{
  let set = new Set([1, 2, 3]);

  // Case 1: Input is empty.
  assertSetContainsExactOrderedItems(set.difference(new Set([])), [1, 2, 3]);

  // Case 2: Input has fewer elements.
  assertSetContainsExactOrderedItems(set.difference(new Set([1, 2])), [3]);
  assertSetContainsExactOrderedItems(set.difference(new Set([2, 1])), [3]);
  assertSetContainsExactOrderedItems(set.difference(new Set([11, 2])), [1, 3]);
  assertSetContainsExactOrderedItems(set.difference(new Set([2, 11])), [1, 3]);

  // Case 3: Input has same number of elements.
  assertSetContainsExactOrderedItems(set.difference(new Set([1, 2, 3])), []);
  assertSetContainsExactOrderedItems(set.difference(new Set([2, 3, 1])), []);
  assertSetContainsExactOrderedItems(set.difference(new Set([3, 2, 1])), []);
  assertSetContainsExactOrderedItems(set.difference(new Set([11, 2, 3])), [1]);
  assertSetContainsExactOrderedItems(set.difference(new Set([2, 3, 11])), [1]);
  assertSetContainsExactOrderedItems(set.difference(new Set([3, 2, 11])), [1]);

  // Case 4: Input has more elements.
  assertSetContainsExactOrderedItems(set.difference(new Set([2, 3, 4, 5])), [1]);
  assertSetContainsExactOrderedItems(set.difference(new Set([4, 5, 2, 3])), [1]);
  assertSetContainsExactOrderedItems(set.difference(new Set([5, 4, 3, 2])), [1]);
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

    assertSetContainsExactOrderedItems(new Set(keys).difference(setLike), []);
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
        throw new Error("Unexpected call to |has| method");
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

    assertSetContainsExactOrderedItems(new Set(keys).difference(setLike), []);
  }
}

// Test result set order when the this-value was modified.
{
  let originalKeys = null;

  let setLike = {
    size: 100,
    has(v) {
      if (!originalKeys) {
        assertSetContainsExactOrderedItems(set, [1, 2, 3, 4]);

        originalKeys = [...set.keys()];

        // Remove all existing items.
        set.clear();

        // Add new keys 11 and 22.
        set.add(11);
        set.add(22);
      }

      // |has| is called exactly once for each key.
      assert.sameValue(originalKeys.includes(v), true);

      originalKeys.splice(originalKeys.indexOf(v), 1);

      return true;
    },
    keys() {
      throw new Error("Unexpected call to |keys| method");
    },
  };

  let set = new Set([1, 2, 3, 4]);

  assertSetContainsExactOrderedItems(set.difference(setLike), []);
  assertSetContainsExactOrderedItems(set, [11, 22]);
  assert.compareArray(originalKeys, []);
}
{
  let setLike = {
    size: 0,
    has() {
      throw new Error("Unexpected call to |has| method");
    },
    *keys() {
      assertSetContainsExactOrderedItems(set, [1, 2, 3, 4]);

      let originalKeys = [...set.keys()];

      // Remove all existing items.
      set.clear();

      // Add new keys 11 and 22.
      set.add(11);
      set.add(22);

      // Yield the original keys of |set|.
      yield* originalKeys;
    },
  };

  let set = new Set([1, 2, 3, 4]);

  assertSetContainsExactOrderedItems(set.difference(setLike), []);
  assertSetContainsExactOrderedItems(set, [11, 22]);
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
  assertSetContainsExactOrderedItems(set.difference(other), []);
} finally {
  Object.defineProperty(Set.prototype, "has", SetPrototypeHas);
  Object.defineProperty(Set.prototype, "keys", SetPrototypeKeys);
  Object.defineProperty(Set.prototype, "size", SetPrototypeSize);
}

