// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: >
  Length property is clamped to 2^53-1, test with indices near 2^53-1 and negative indices
  and a proxy to an array.
info: |
  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let relativeStart be ? ToInteger(start).
  4. If relativeStart < 0, let k be max((len + relativeStart), 0);
     else let k be min(relativeStart, len).
  5. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToInteger(end).
  6. If relativeEnd < 0, let final be max((len + relativeEnd), 0);
     else let final be min(relativeEnd, len).
  ...
includes: [compareArray.js]
features: [exponentiation]
---*/

var array = [];
array["9007199254740988"] = "9007199254740988";
array["9007199254740989"] = "9007199254740989";
array["9007199254740990"] = "9007199254740990";
array["9007199254740991"] = "9007199254740991";

// Create a proxy to an array object, so IsArray returns true, but we can still
// return a length value exceeding the integer limit.
var proxy = new Proxy(array, {
  get(t, pk, r) {
    if (pk === "length")
      return 2 ** 53 + 2;
    return Reflect.get(t, pk, r);
  }
});

var result = Array.prototype.slice.call(proxy, 9007199254740989);
assert.compareArray(result, ["9007199254740989", "9007199254740990"],
  'The value of result is expected to be ["9007199254740989", "9007199254740990"]');

var result = Array.prototype.slice.call(proxy, 9007199254740989, 9007199254740990);
assert.compareArray(result, ["9007199254740989"],
  'The value of result is expected to be ["9007199254740989"]');

var result = Array.prototype.slice.call(proxy, 9007199254740989, 9007199254740996);
assert.compareArray(result, ["9007199254740989", "9007199254740990"],
  'The value of result is expected to be ["9007199254740989", "9007199254740990"]');

var result = Array.prototype.slice.call(proxy, -2);
assert.compareArray(result, ["9007199254740989", "9007199254740990"],
  'The value of result is expected to be ["9007199254740989", "9007199254740990"]');

var result = Array.prototype.slice.call(proxy, -2, -1);
assert.compareArray(result, ["9007199254740989"],
  'The value of result is expected to be ["9007199254740989"]');
