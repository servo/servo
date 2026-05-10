// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object-initializer-runtime-semantics-propertydefinitionevaluation
description: >
  Proxy's "get" trap is not invoked for non-enumerable keys.
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
      ii. If desc is not undefined and desc.[[Enumerable]] is true, then
        1. Let propValue be ? Get(from, nextKey).
        2. Perform ! CreateDataPropertyOrThrow(target, nextKey, propValue).

  [[OwnPropertyKeys]] ( )

  [...]
  7. Let trapResultArray be ? Call(trap, handler, « target »).
  8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).
  [...]
  23. Return trapResult.
features: [object-spread, Proxy, Symbol]
includes: [compareArray.js, propertyHelper.js]
---*/

var VALUE_LITERAL = "VALUE_LITERAL";
var VALUE_GOPD = "VALUE_GOPD";
var VALUE_GET = "VALUE_GET";

var dontEnumSymbol = Symbol("dont_enum_symbol");
var enumerableSymbol = Symbol("enumerable_symbol");

var dontEnumKeys = [dontEnumSymbol, "dontEnumString", "0"];
var enumerableKeys = [enumerableSymbol, "enumerableString", "1"];
var ownKeysResult = [...dontEnumKeys, ...enumerableKeys];

var getOwnKeys = [];
var getKeys = [];
var proxy = new Proxy({}, {
  getOwnPropertyDescriptor: function(_target, key) {
    getOwnKeys.push(key);
    var isEnumerable = enumerableKeys.indexOf(key) !== -1;
    return {value: VALUE_GOPD, writable: false, enumerable: isEnumerable, configurable: true};
  },
  get: function(_target, key) {
    getKeys.push(key);
    return VALUE_GET;
  },
  ownKeys: function() {
    return ownKeysResult;
  },
});

var result = {[enumerableSymbol]: VALUE_LITERAL, enumerableString: VALUE_LITERAL, [1]: VALUE_LITERAL, ...proxy};
assert.compareArray(getOwnKeys, ownKeysResult);
assert.compareArray(getKeys, enumerableKeys);

verifyProperty(result, enumerableSymbol, {value: VALUE_GET, writable: true, enumerable: true, configurable: true});
verifyProperty(result, "enumerableString", {value: VALUE_GET, writable: true, enumerable: true, configurable: true});
verifyProperty(result, "1", {value: VALUE_GET, writable: true, enumerable: true, configurable: true});
