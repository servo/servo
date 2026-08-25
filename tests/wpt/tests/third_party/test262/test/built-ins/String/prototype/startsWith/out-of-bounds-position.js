// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Returns false if searchLength + start position is greater than string length.
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  ...
  11. Let len be the number of elements in S.
  12. Let start be min(max(pos, 0), len).
  13. Let searchLength be the number of elements in searchStr.
  14. If searchLength+start is greater than len, return false.
  ...
---*/

var str = 'The future is cool!';

assert.sameValue(
  str.startsWith('!', str.length), false,
  'str.startsWith("!", str.length) returns false'
);

assert.sameValue(
  str.startsWith('!', 100), false,
  'str.startsWith("!", 100) returns false'
);

assert.sameValue(
  str.startsWith('!', Infinity), false,
  'str.startsWith("!", Infinity) returns false'
);

assert(
  str.startsWith('The future', -1),
  'position argument < 0 will search from the start of the string (-1)'
);

assert(
  str.startsWith('The future', -Infinity),
  'position argument < 0 will search from the start of the string (-Infinity)'
);
