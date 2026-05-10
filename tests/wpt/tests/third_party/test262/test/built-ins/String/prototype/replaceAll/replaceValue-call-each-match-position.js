// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  If replaceValue is a function, it's called for each matching position
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
  return 'z';
};

var searchValue = new String('ab c');

var obj = new String('ab c ab cdab cab c');

var result = obj.replaceAll(searchValue, replaceValue);
assert.sameValue(calls.length, 4);
assert.sameValue(result, 'z zdzz');

var str = obj.toString();

assert.compareArray(calls[0], [t, 'ab c', 0, str]);
assert.compareArray(calls[1], [t, 'ab c', 5, str]);
assert.compareArray(calls[2], [t, 'ab c', 10, str]);
assert.compareArray(calls[3], [t, 'ab c', 14, str]);
