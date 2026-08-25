// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@search
description: >
  `currentLastIndex` value is compared using SameValue. 
info: |
  RegExp.prototype [ @@search ] ( string )

  [...]
  6. Let result be ? RegExpExec(rx, S).
  7. Let currentLastIndex be ? Get(rx, "lastIndex").
  8. If SameValue(currentLastIndex, previousLastIndex) is false, then
    a. Perform ? Set(rx, "lastIndex", previousLastIndex, true).
  [...]
features: [Symbol.search]
---*/

var re = /(?:)/;
re.exec = function() {
  re.lastIndex = -0;
  return null;
};

assert.sameValue(re[Symbol.search](""), -1);
assert.sameValue(re.lastIndex, 0);
