// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-enumerate-object-properties
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  EnumerateObjectProperties ( O )

  EnumerateObjectProperties must obtain the own property keys of the target object
  by calling its [[OwnPropertyKeys]] internal method. Property attributes of the
  target object must be obtained by calling its [[GetOwnProperty]] internal method.

  OrdinaryOwnPropertyKeys ( O )

  [...]
  3. For each own property key P of O that is a String but is not an array index,
  in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  [...]
  5. Return keys.
includes: [compareArray.js]
---*/

var obj = {};
obj.a = 1;
obj.b = 2;
Object.defineProperty(obj, "a", {value: 11});
var objKeys = [];
for (var objKey in obj) {
  objKeys.push(objKey);
}
assert.compareArray(objKeys, ["a", "b"]);

var arr = [];
Object.defineProperty(arr, "a", {
  get: function() {},
  enumerable: true,
  configurable: true,
})
arr.b = 2;
Object.defineProperty(arr, "a", {
  get: function() {},
});
var arrKeys = [];
for (var arrKey in arr) {
  arrKeys.push(arrKey);
}
assert.compareArray(arrKeys, ["a", "b"]);
