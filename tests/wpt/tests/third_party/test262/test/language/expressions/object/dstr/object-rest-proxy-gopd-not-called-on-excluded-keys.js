// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-destructuring-binding-patterns-runtime-semantics-restbindinginitialization
description: >
  Proxy's "getOwnPropertyDescriptor" trap is not invoked for excluded keys.
info: |
  BindingRestProperty : ... BindingIdentifier

  [...]
  3. Perform ? CopyDataProperties(restObj, value, excludedNames).

  CopyDataProperties ( target, source, excludedItems )

  [...]
  5. Let keys be ? from.[[OwnPropertyKeys]]().
  6. For each element nextKey of keys in List order, do
    b. For each element e of excludedItems, do
      i. If SameValue(e, nextKey) is true, then
        1. Set excluded to true.
    c. If excluded is false, then
      i. Let desc be ? from.[[GetOwnProperty]](nextKey).

  [[OwnPropertyKeys]] ( )

  [...]
  7. Let trapResultArray be ? Call(trap, handler, « target »).
  8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
  [...]
  23. Return trapResult.
features: [object-rest, destructuring-binding, Proxy, Symbol]
includes: [compareArray.js]
---*/

var excludedSymbol = Symbol("excluded_symbol");
var includedSymbol = Symbol("included_symbol");

var excludedKeys = [excludedSymbol, "excludedString", "0"];
var includedKeys = [includedSymbol, "includedString", "1"];
var ownKeysResult = [...excludedKeys, ...includedKeys];

var getOwnKeys = [];
var proxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    getOwnKeys.push(key);
  },
  ownKeys: function() {
    return ownKeysResult;
  },
});

var {[excludedSymbol]: _, excludedString, 0: excludedIndex, ...rest} = proxy;
assert.compareArray(getOwnKeys, includedKeys);
