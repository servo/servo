// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  Integer coercion of "index" property of the value returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    e. Let position be ? ToInteger(? Get(result, "index")).
    [...]
    k. If functionalReplace is true, then
      i. Let replacerArgs be « matched ».
      ii. Append in list order the elements of captures to the end of the List replacerArgs.
      iii. Append position and S to replacerArgs.
      [...]
      v. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
features: [Symbol.replace]
---*/

var r = /./;
var coercibleIndex = {
  length: 1,
  0: '',
  index: {
    valueOf: function() {
      return 2.9;
    },
  },
};
r.exec = function() {
  return coercibleIndex;
};

var replacer = function(_matched, position) {
  return position;
};

assert.sameValue(r[Symbol.replace]('abcd', replacer), 'ab2cd');
