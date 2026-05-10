// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@split
description: >
  Side-effects in ToUint32 may recompile the regular expression.
info: |
  21.2.5.11 RegExp.prototype [ @@split ] ( string, limit )
    ...
    10. Let splitter be ? Construct(C, « rx, newFlags »).
    ...
    13. If limit is undefined, let lim be 2^32-1; else let lim be ? ToUint32(limit).
    ...

features: [Symbol.split]
---*/

var regExp = /a/;
var limit = {
  valueOf: function() {
    regExp.compile("b");
    return -1;
  }
};

var result = regExp[Symbol.split]("abba", limit);

assert.sameValue(result.length, 3);
assert.sameValue(result[0], "");
assert.sameValue(result[1], "bb");
assert.sameValue(result[2], "");
