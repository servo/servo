// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@search
description: >
  `previousLastIndex` value is compared using SameValue. 
info: |
  RegExp.prototype [ @@search ] ( string )

  [...]
  4. Let previousLastIndex be ? Get(rx, "lastIndex").
  5. If SameValue(previousLastIndex, 0) is false, then
    a. Perform ? Set(rx, "lastIndex", 0, true).
  6. Let result be ? RegExpExec(rx, S).
  [...]
features: [Symbol.search]
---*/

var re = /(?:)/;
var execLastIndex;

re.lastIndex = -0;
re.exec = function() {
  execLastIndex = re.lastIndex;
  return null;
};

assert.sameValue(re[Symbol.search](""), -1);
assert.sameValue(execLastIndex, 0);
