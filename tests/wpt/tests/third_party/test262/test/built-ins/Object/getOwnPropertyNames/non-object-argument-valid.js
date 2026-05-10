// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.getownpropertynames
description: >
    Object.getOwnPropertyNames called with a valid non-object value
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
  Object.getOwnPropertyNames(true), [],
  'Object.getOwnPropertyNames(true) must return []'
);
assert.compareArray(
  Object.getOwnPropertyNames(false), [],
  'Object.getOwnPropertyNames(false) must return []'
);
assert.compareArray(
  Object.getOwnPropertyNames(1), [],
  'Object.getOwnPropertyNames(1) must return []'
);
assert.compareArray(
  Object.getOwnPropertyNames(0), [],
  'Object.getOwnPropertyNames(0) must return []'
);
assert.compareArray(
  Object.getOwnPropertyNames(""), ["length"],
  'Object.getOwnPropertyNames("") must return ["length"]'
);
assert.compareArray(
  Object.getOwnPropertyNames(Symbol()), [],
  'Object.getOwnPropertyNames(Symbol()) must return []'
);
