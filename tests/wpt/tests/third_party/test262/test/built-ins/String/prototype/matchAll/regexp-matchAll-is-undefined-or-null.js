// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when regexp[@@matchAll] is undefined or null
info: |
  String.prototype.matchAll ( regexp )
    1. Let O be ? RequireObjectCoercible(this value).
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
      b. If matcher is not undefined, then
        [...]
    3. Let S be ? ToString(O).
    4. Let rx be ? RegExpCreate(R, "g").
    5. Return ? Invoke(rx, @@matchAll, « S »).
features: [Symbol.matchAll, String.prototype.matchAll]
---*/

var regexp = /./g;
var callCount = 0;
var arg;
var obj = {};
var str = 'abc';
RegExp.prototype[Symbol.matchAll] = function(string) {
  arg = string;
  callCount++;
  return obj;
};

regexp[Symbol.matchAll] = undefined;
assert.sameValue(str.matchAll(regexp), obj);
assert.sameValue(arg, str);
assert.sameValue(callCount, 1);

regexp[Symbol.matchAll] = null;
assert.sameValue(str.matchAll(regexp), obj);
assert.sameValue(arg, str);
assert.sameValue(callCount, 2);
