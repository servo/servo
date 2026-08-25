// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.values
description: >
  Property names are returned in ascending chronological order of creation
  that is unaffected by [[DefineOwnProperty]].
info: |
  Object.values ( O )

  [...]
  2. Let nameList be ? EnumerableOwnPropertyNames(obj, value).
  3. Return CreateArrayFromList(nameList).

  EnumerableOwnPropertyNames ( O, kind )

  [...]
  2. Let ownKeys be ? O.[[OwnPropertyKeys]]().
  [...]

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
Object.defineProperty(obj, "a", {
  get: function() {},
  enumerable: true,
  configurable: true,
});
obj.b = "b";
Object.defineProperty(obj, "a", {
  get: function() {
    return "a";
  },
});
assert.compareArray(Object.values(obj), ["a", "b"]);

var proxy = new Proxy({}, {});
Object.defineProperty(proxy, "a", {
  get: function() {},
  enumerable: true,
  configurable: true,
});
proxy.b = "b";
Object.defineProperty(proxy, "a", {value: "a"});
assert.compareArray(Object.values(proxy), ["a", "b"]);
