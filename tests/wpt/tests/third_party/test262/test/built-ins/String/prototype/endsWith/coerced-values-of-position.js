// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Returns based on coerced values of endPosition.
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

assert(str.endsWith('', NaN), 'NaN coerced to 0');
assert(str.endsWith('', null), 'null coerced to 0');
assert(str.endsWith('', false), 'false coerced to 0');
assert(str.endsWith('', ''), '"" coerced to 0');
assert(str.endsWith('', '0'), '"0" coerced to 0');
assert(str.endsWith('', undefined), 'undefined coerced to 0');

assert(str.endsWith('The future', 10.4), '10.4 coerced to 10');
assert(str.endsWith('T', true), 'true coerced to 1');

assert(str.endsWith('The future', '10'), '"10" coerced to 10');
