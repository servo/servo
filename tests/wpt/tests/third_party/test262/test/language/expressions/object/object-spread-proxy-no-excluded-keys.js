// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-propertydefinitionevaluation
description: >
  Proxy's "getOwnPropertyDescriptor" trap is invoked for all keys.
info: |
  PropertyDefinition : ... AssignmentExpression

  [...]
  3. Let excludedNames be a new empty List.
  4. Return ? CopyDataProperties(object, fromValue, excludedNames).

  CopyDataProperties ( target, source, excludedItems )

  [...]
  5. Let keys be ? from.[[OwnPropertyKeys]]().
  6. For each element nextKey of keys in List order, do
    [...]
    c. If excluded is false, then
      i. Let desc be ? from.[[GetOwnProperty]](nextKey).

  [[OwnPropertyKeys]] ( )

  [...]
  7. Let trapResultArray be ? Call(trap, handler, « target »).
  8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
  [...]
  23. Return trapResult.
features: [object-spread, Proxy, Symbol]
includes: [compareArray.js]
---*/

var sym = Symbol();
var getOwnKeys = [];
var ownKeysResult = [sym, "foo", "0"];
var proxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    getOwnKeys.push(key);
  },
  ownKeys: function() {
    return ownKeysResult;
  },
});

({[sym]: 0, foo: 0, [0]: 0, ...proxy});
assert.compareArray(getOwnKeys, ownKeysResult);
