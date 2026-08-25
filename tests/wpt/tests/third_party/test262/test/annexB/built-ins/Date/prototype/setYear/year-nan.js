// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: Behavior when year value coerces to NaN
info: |
    [...]
    3. Let y be ? ToNumber(year).
    4. If y is NaN, set the [[DateValue]] internal slot of this Date object to
       NaN and return NaN.
features: [Symbol]
---*/

var date;

date = new Date(0);
assert.sameValue(date.setYear(), NaN, 'return value (no argument)');
assert.sameValue(
  date.valueOf(), NaN, '[[DateValue]] internal slot (no argument)'
);

date = new Date(0);
assert.sameValue(date.setYear(NaN), NaN, 'return value (literal NaN)');
assert.sameValue(
  date.valueOf(), NaN, '[[DateValue]] internal slot (literal NaN)'
);

date = new Date(0);
assert.sameValue(
  date.setYear('not a number'), NaN, 'return value (NaN from ToNumber)'
);
assert.sameValue(
  date.valueOf(), NaN, '[[DateValue]] internal slot (NaN from ToNumber)'
);
