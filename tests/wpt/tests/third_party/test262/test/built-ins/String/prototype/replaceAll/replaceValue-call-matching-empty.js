// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  replaceValue can be called for matching position of an empty string
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  5. Let functionalReplace be IsCallable(replaceValue).
  ...
  14. For each position in matchPositions, do
    a. If functionalReplace is true, then
      i. Let replacement be ? ToString(? Call(replaceValue, undefined, « searchString, position, string »).
features: [String.prototype.replaceAll]
includes: [compareArray.js]
---*/

var t = (function() { return this; })();

var calls = [];
var replaceValue = function(...args) {
  calls.push([this, ...args]);
  return 'abc';
};

var searchValue = new String('');

var obj = new String('');

var result = obj.replaceAll(searchValue, replaceValue);
assert.sameValue(calls.length, 1);
assert.sameValue(result, 'abc');

var str = obj.toString();

assert.compareArray(calls[0], [t, '', 0, str]);
