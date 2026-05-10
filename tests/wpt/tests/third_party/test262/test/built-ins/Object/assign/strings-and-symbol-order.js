// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.assign
description: >
  Symbol-valued properties are copied after String-valued properties.
info: |
  19.1.2.1 Object.assign ( target, ...sources )

  ...
  4. For each element nextSource of sources, in ascending index order, do
    a. ...
    b. Else,
      i. Let from be ! ToObject(nextSource).
      ii. Let keys be ? from.[[OwnPropertyKeys]]().
    c. For each element nextKey of keys in List order, do
    ...
  ...

  9.1.11.1 OrdinaryOwnPropertyKeys ( O )

  ...
  3. For each own property key P of O that is a String but is not an integer index,
     in ascending chronological order of property creation, do
    a. Add P as the last element of keys.
  4. For each own property key P of O that is a Symbol, in ascending chronological
     order of property creation, do
    a. Add P as the last element of keys.
  ...

includes: [compareArray.js]
---*/

var log = [];

var sym1 = Symbol("x");
var sym2 = Symbol("y");

var source = {};

Object.defineProperty(source, sym1, {
    get: function(){ log.push("get sym(x)") },
    enumerable: true, configurable: true,
});
Object.defineProperty(source, "a", {
    get: function(){ log.push("get a") },
    enumerable: true, configurable: true,
});
Object.defineProperty(source, sym2, {
    get: function(){ log.push("get sym(y)") },
    enumerable: true, configurable: true,
});
Object.defineProperty(source, "b", {
    get: function(){ log.push("get b") },
    enumerable: true, configurable: true,
});

var target = Object.assign({}, source);

assert.compareArray(log, ["get a", "get b", "get sym(x)", "get sym(y)"]);
