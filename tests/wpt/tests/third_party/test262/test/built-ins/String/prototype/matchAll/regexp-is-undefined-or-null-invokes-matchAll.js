// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when regexp is null or undefined
info: |
  String.prototype.matchAll ( regexp )
    1. Let O be ? RequireObjectCoercible(this value).
    2. If regexp is neither undefined nor null, then
      [...]
    3. Let S be ? ToString(O).
    4. Let rx be ? RegExpCreate(R, "g").
    5. Return ? Invoke(rx, @@matchAll, « S »).
features: [String.prototype.matchAll]
---*/

var callCount = 0;
var obj = {};
RegExp.prototype[Symbol.matchAll] = function() {
  callCount++;
  return obj;
};

assert.sameValue('a'.matchAll(null), obj);
assert.sameValue(callCount, 1);

assert.sameValue(''.matchAll(undefined), obj);
assert.sameValue(callCount, 2);

