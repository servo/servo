// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Returns false if searchString is not found with a given position.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  ...
  10. If endPosition is undefined, let pos be len, else let pos be
  ToInteger(endPosition).
  11. ReturnIfAbrupt(pos).
  12. Let end be min(max(pos, 0), len).
  13. Let searchLength be the number of elements in searchStr.
  14. Let start be end - searchLength.
  15. If start is less than 0, return false.
  16. If the sequence of elements of S starting at start of length searchLength
  is the same as the full element sequence of searchStr, return true.
  17. Otherwise, return false.
  ...
features: [String.prototype.endsWith]
---*/

var str = 'The future is cool!';

assert.sameValue(
  str.endsWith('is cool!', str.length - 1), false,
  'str.endsWith("is cool!", str.length - 1) === false'
);

assert.sameValue(
  str.endsWith('!', 1), false,
  'str.endsWith("!", 1) === false'
);
