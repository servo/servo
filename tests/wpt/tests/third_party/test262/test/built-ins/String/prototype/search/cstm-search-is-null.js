// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.search
description: >
  If regexp's Symbol.search property is null, no error is thrown.
info: |
  String.prototype.search ( regexp )

  [...]
  2. If regexp is neither undefined nor null, then
    a. Let searcher be ? GetMethod(regexp, @@search).
    b. If searcher is not undefined, then
      [...]
  [...]
  5. Return ? Invoke(rx, @@search, « string »).

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
features: [Symbol.search]
---*/

var regexp = {};
regexp[Symbol.search] = null;
regexp.toString = function() { return "\\d"; };
regexp.valueOf = function() { throw new Test262Error("should not be called"); };

assert.sameValue("abc".search(regexp), -1);
assert.sameValue("ab3c".search(regexp), 2);
