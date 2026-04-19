// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  String coercion of "groups" property of the value returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    j. Let namedCaptures be ? Get(result, "groups").
    k. If functionalReplace is true, then
      [...]
    l. Else,
      i. If namedCaptures is not undefined, then
        1. Set namedCaptures to ? ToObject(namedCaptures).
      ii. Let replacement be ? GetSubstitution(matched, S, position, captures, namedCaptures, replaceValue).
features: [Symbol.replace, regexp-named-groups]
---*/

var r = /./;
var coercibleValue = {
  length: 1,
  0: 'b',
  index: 1,
  groups: '123',
};

r.exec = function() {
  return coercibleValue;
};

assert.sameValue(r[Symbol.replace]('ab', '[$<length>]'), 'a[3]');
