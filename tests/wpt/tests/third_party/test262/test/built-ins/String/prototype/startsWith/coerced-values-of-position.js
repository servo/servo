// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Returns based on coerced values of position.
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  ...
  9. Let pos be ToInteger(position). (If position is undefined, this step
  produces the value 0).
  10. ReturnIfAbrupt(pos).
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

assert(str.startsWith('The future', NaN), 'NaN coerced to 0');
assert(str.startsWith('The future', null), 'null coerced to 0');
assert(str.startsWith('The future', false), 'false coerced to 0');
assert(str.startsWith('The future', ''), '"" coerced to 0');
assert(str.startsWith('The future', '0'), '"0" coerced to 0');
assert(str.startsWith('The future', undefined), 'undefined coerced to 0');
assert(str.startsWith('The future', 0.4), '0.4 coerced to 0');

assert.sameValue(
  str.startsWith('The future', true), false,
  'true coerced to 1'
);
assert.sameValue(str.startsWith('The future', '1'), false, '"1" coerced to 1');
assert.sameValue(str.startsWith('The future', 1.4), false, '1.4 coerced to 1');
