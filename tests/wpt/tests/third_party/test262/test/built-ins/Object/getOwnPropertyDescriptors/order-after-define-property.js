// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.getownpropertydescriptors
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.getOwnPropertyDescriptors ( O )

  [...]
  2. Let ownKeys be ? obj.[[OwnPropertyKeys]]().
  3. Let descriptors be ! OrdinaryObjectCreate(%Object.prototype%).
  4. For each element key of ownKeys in List order, do
    [...]
    c. If descriptor is not undefined,
    perform ! CreateDataPropertyOrThrow(descriptors, key, descriptor).
  5. Return descriptors.

  OrdinaryOwnPropertyKeys ( O )

  [...]
  3. For each own property key P of O that is a String but is not an array index,
  in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  4. For each own property key P of O that is a Symbol, in ascending
  chronological order of property creation, do
    a. Add P as the last element of keys.
  5. Return keys.
features: [Symbol, Reflect]
includes: [compareArray.js]
---*/

var obj = {};
var symA = Symbol("a");
var symB = Symbol("b");
obj[symA] = 1;
obj[symB] = 2;
Object.defineProperty(obj, symA, {configurable: false});
var objDescs = Object.getOwnPropertyDescriptors(obj);
assert.compareArray(Reflect.ownKeys(objDescs), [symA, symB]);

var re = /(?:)/g;
re.a = 1;
Object.defineProperty(re, "lastIndex", {value: 2});
var reDescs = Object.getOwnPropertyDescriptors(re);
assert.compareArray(Reflect.ownKeys(reDescs), ["lastIndex", "a"]);
