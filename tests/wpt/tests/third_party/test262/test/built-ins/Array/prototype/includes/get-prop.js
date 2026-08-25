// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: get array-like indexed properties
info: |
  22.1.3.11 Array.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  7. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
  ...
includes: [compareArray.js]
features: [Proxy, Array.prototype.includes]
---*/

var calls;

var obj = {};

var p = new Proxy(obj, {
  get: function(_, key) {
    calls.push(key);

    if (key === "length") {
      return 4;
    }

    return key * 10;
  }
});

calls = [];
assert.sameValue(
  [].includes.call(p, 42),
  false,
  '[].includes.call("new Proxy(obj, {get: function(_, key) {calls.push(key); if (key === "length") {return 4;} return key * 10;}})", 42) must return false'
);
assert.compareArray(calls, ["length", "0", "1", "2", "3"],
  'The value of calls is expected to be ["length", "0", "1", "2", "3"]'
);

calls = [];
assert.sameValue([].includes.call(p, 10), true, '[].includes.call("new Proxy(obj, {get: function(_, key) {calls.push(key); if (key === "length") {return 4;} return key * 10;}})", 10) must return true');
assert.compareArray(calls, ["length", "0", "1"], 'The value of calls is expected to be ["length", "0", "1"]');
