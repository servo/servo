// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: Clipping of new time value
info: |
    [...]
    9. Set the [[DateValue]] internal slot of this Date object to
       TimeClip(date).
    10. Return the value of the [[DateValue]] internal slot of this Date
        object.
---*/

var date;

date = new Date(1970, 8, 10, 0, 0, 0, 0);

assert.notSameValue(
  date.setYear(275760), NaN, 'method return value (valid date)'
);
assert.notSameValue(
  date.valueOf(), NaN, '[[DateValue]] internal slot (valid date)'
);

date = new Date(1970, 8, 14, 0, 0, 0, 0);

assert.sameValue(
  date.setYear(275760), NaN, 'method return value (invalid date)'
);
assert.sameValue(
  date.valueOf(), NaN, '[[DateValue]] internal slot (invalid date)'
);
