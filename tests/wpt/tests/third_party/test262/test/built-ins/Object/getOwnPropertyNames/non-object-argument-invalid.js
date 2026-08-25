// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.getownpropertynames
description: >
    Object.getOwnPropertyNames called with an invalid non-object value
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
---*/

let count = 0;

assert.throws(TypeError, () => {
  count++;
  Object.getOwnPropertyNames(undefined);
}, '`Object.getOwnPropertyNames(undefined)` throws TypeError');

assert.throws(TypeError, () => {
  count++;
  Object.getOwnPropertyNames(null);
}, '`Object.getOwnPropertyNames(null)` throws TypeError');

assert.sameValue(count, 2, 'The value of `count` is 2');
