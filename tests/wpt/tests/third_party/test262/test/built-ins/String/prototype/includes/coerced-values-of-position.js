// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.7
description: >
  Returns based on coerced values of position.
info: |
  21.1.3.7 String.prototype.includes ( searchString [ , position ] )

  ...
  9. Let pos be ToInteger(position). (If position is undefined, this step
  produces the value 0).
  10. ReturnIfAbrupt(pos).
  11. Let len be the number of elements in S.
  12. Let start be min(max(pos, 0), len).
  13. Let searchLen be the number of elements in searchStr.
  14. If there exists any integer k not smaller than start such that k +
  searchLen is not greater than len, and for all nonnegative integers j less
  than searchLen, the code unit at index k+j of S is the same as the code unit
  at index j of searchStr, return true; but if there is no such integer k,
  return false.
  ...
features: [String.prototype.includes]
---*/

var str = 'The future is cool!';

assert(str.includes('The future', NaN), 'NaN coerced to 0');
assert(str.includes('The future', null), 'null coerced to 0');
assert(str.includes('The future', false), 'false coerced to 0');
assert(str.includes('The future', ''), '"" coerced to 0');
assert(str.includes('The future', '0'), '"0" coerced to 0');
assert(str.includes('The future', undefined), 'undefined coerced to 0');
assert(str.includes('The future', 0.4), '0.4 coerced to 0');

assert(str.includes('The future', -1));

assert.sameValue(str.includes('The future', true), false, 'true coerced to 1');
assert.sameValue(str.includes('The future', '1'), false, '"1" coerced to 1');
assert.sameValue(str.includes('The future', 1.4), false, '1.4 coerced to 1');
