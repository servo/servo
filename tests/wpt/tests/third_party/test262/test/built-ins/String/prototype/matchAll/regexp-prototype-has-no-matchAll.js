// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when @@matchAll is removed from RegExp's prototype
info: |
  String.prototype.matchAll ( regexp )
    1. Let O be ? RequireObjectCoercible(this value).
    2. If regexp is neither undefined nor null, then
      a. Let matcher be ? GetMethod(regexp, @@matchAll).
      b. If matcher is not undefined, then
        [...]
    [...]
    4. Let rx be ? RegExpCreate(R, "g").
    5. Return ? Invoke(rx, @@matchAll, « S »).

features: [Symbol.matchAll, String.prototype.matchAll]
---*/

assert.sameValue(typeof String.prototype.matchAll, "function");

delete RegExp.prototype[Symbol.matchAll];
var str = '/a/g*/b/g';

assert.throws(TypeError, function() {
  str.matchAll(/\w/g);
});
