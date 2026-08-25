// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.13
description: >
  Returns a String made from n copies of the original String appended together.
info: |
  21.1.3.13 String.prototype.repeat ( count )

  8. Let T be a String value that is made from n copies of S appended together.
  If n is 0, T is the empty String.
  9. Return T.
---*/

var str = 'abc';
assert.sameValue(str.repeat(1), str, 'str.repeat(1) === str');
assert.sameValue(str.repeat(3), 'abcabcabc', 'str.repeat(3) === "abcabcabc"');

str = '';
var i = 0;
var count = 10000;

while (i < count) {
  str += '.';
  i++;
}

assert.sameValue('.'.repeat(count), str);
