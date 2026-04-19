// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.getownpropertysymbols
description: >
    Object.getOwnPropertySymbols called with a valid non-object value
info: |
  GetOwnPropertyKeys ( O, type )

  Let obj be ? ToObject(O).
  Let keys be ? obj.[[OwnPropertyKeys]]().
  Let nameList be a new empty List.
  For each element nextKey of keys, do
    If Type(nextKey) is Symbol and type is symbol or Type(nextKey) is String and type is string, then
      Append nextKey as the last element of nameList.
  Return CreateArrayFromList(nameList).

features: [Symbol]
includes: [compareArray.js]
---*/

assert.compareArray(
  Object.getOwnPropertySymbols(true), [],
  'Object.getOwnPropertySymbols(true) must return []'
);
assert.compareArray(
  Object.getOwnPropertySymbols(false), [],
  'Object.getOwnPropertySymbols(false) must return []'
);
assert.compareArray(
  Object.getOwnPropertySymbols(1), [],
  'Object.getOwnPropertySymbols(1) must return []'
);
assert.compareArray(
  Object.getOwnPropertySymbols(0), [],
  'Object.getOwnPropertySymbols(0) must return []'
);
assert.compareArray(
  Object.getOwnPropertySymbols(""), [],
  'Object.getOwnPropertySymbols("") must return []'
);
assert.compareArray(
  Object.getOwnPropertySymbols(Symbol()), [],
  'Object.getOwnPropertySymbols(Symbol()) must return []'
);
