// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  Type coercion of "length" property of the value returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    a. Let nCaptures be ? LengthOfArrayLike(result).
    [...]
features: [Symbol.replace]
---*/

var r = /./;
var coercibleIndex = {
  length: {
    valueOf: function() {
      return 3.9;
    },
  },
  0: '',
  1: 'foo',
  2: 'bar',
  3: 'baz',
  index: 0,
};
r.exec = function() {
  return coercibleIndex;
};

assert.sameValue(r[Symbol.replace]('', '$1$2$3'), 'foobar$3');
