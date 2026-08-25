// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Returns false if searchString is not found.
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  ...
  11. Let len be the number of elements in S.
  12. Let start be min(max(pos, 0), len).
  13. Let searchLength be the number of elements in searchStr.
  14. If searchLength+start is greater than len, return false.
  15. If the sequence of elements of S starting at start of length searchLength
  is the same as the full element sequence of searchStr, return true.
  16. Otherwise, return false.
  ...
---*/

var str = 'The future is cool!';

assert.sameValue(
  str.startsWith('Flash'), false,
  'str.startsWith("Flash") === false'
);
assert.sameValue(
  str.startsWith('THE FUTURE'), false,
  'startsWith is case sensitive'
);
assert.sameValue(
  str.startsWith('future is cool!'), false,
  'str.startsWith("future is cool!") === false'
);
