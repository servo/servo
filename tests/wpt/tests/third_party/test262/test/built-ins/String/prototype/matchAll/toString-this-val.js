// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
  Verify ToString is called when regexp[@@matchAll] is undefined or null
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

var regexp = /./;
var callCount = 0;
var arg;
var obj = {};
var toStringResult = 'abc';
var receiver = {
  [Symbol.toPrimitive]: function() {
    callCount++;
    return toStringResult;
  }
};
RegExp.prototype[Symbol.matchAll] = function(string) {
  arg = string;
};

String.prototype.matchAll.call(receiver, null);
assert.sameValue(callCount, 1);
assert.sameValue(arg, toStringResult);

String.prototype.matchAll.call(receiver, undefined);
assert.sameValue(callCount, 2);
assert.sameValue(arg, toStringResult);
