// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.split
description: >
  If separator's Symbol.split property is null, no error is thrown.
info: |
  String.prototype.split ( separator, limit )

  [...]
  2. If separator is neither undefined nor null, then
    a. Let splitter be ? GetMethod(separator, @@split).
    b. If splitter is not undefined, then
      [...]
  [...]
  17. Return A.

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
includes: [compareArray.js]
features: [Symbol.split]
---*/

var separator = {};
separator[Symbol.split] = null;
separator.toString = function() { return "2"; };
separator.valueOf = function() { throw new Test262Error("should not be called"); };

assert.compareArray("a2b2c".split(separator), ["a", "b", "c"]);
assert.compareArray("a2b2c".split(separator, 1), ["a"]);
