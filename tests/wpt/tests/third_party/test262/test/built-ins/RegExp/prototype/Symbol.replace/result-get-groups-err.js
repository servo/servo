// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  Abrupt completion during lookup of "groups"
  property of the value returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    j. Let namedCaptures be ? Get(result, "groups").
features: [Symbol.replace, regexp-named-groups]
---*/

var r = /./;
var coercibleValue = {
  length: 0,
  index: 0,
  get groups() {
    throw new Test262Error();
  },
};

r.exec = function() {
  return coercibleValue;
};

assert.throws(Test262Error, function() {
  r[Symbol.replace]('a', '$<foo>');
});
