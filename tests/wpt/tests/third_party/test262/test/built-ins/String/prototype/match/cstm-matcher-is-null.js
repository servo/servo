// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.match
description: >
  If regexp's Symbol.match property is null, no error is thrown.
info: |
  String.prototype.match ( regexp )

  [...]
  2. If regexp is neither undefined nor null, then
    a. Let matcher be ? GetMethod(regexp, @@match).
    b. If matcher is not undefined, then
      [...]
  [...]
  5. Return ? Invoke(rx, @@match, « S »).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Symbol.match]
---*/

var regexp = {};
regexp[Symbol.match] = null;
regexp.toString = function() { return "\\d"; };
regexp.valueOf = function() { throw new Test262Error("should not be called"); };

assert.sameValue("abc".match(regexp), null);
assert.sameValue("ab3c".match(regexp)[0], "3");
