// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: >
    Behavior when the integer representation of the specified `year` is
    relative to 1900
info: |
    [...]
    5. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yyyy be ToInteger(y) +
       1900.
    [...]
---*/

var date;

date = new Date(1970, 0);
date.setYear(-0.9999999);
assert.sameValue(date.getFullYear(), 1900, 'y = -0.999999');

date = new Date(1970, 0);
date.setYear(-0);
assert.sameValue(date.getFullYear(), 1900, 'y = -0');

date = new Date(1970, 0);
date.setYear(0);
assert.sameValue(date.getFullYear(), 1900, 'y = 0');

date = new Date(1970, 0);
date.setYear(50);
assert.sameValue(date.getFullYear(), 1950, 'y = 50');

date = new Date(1970, 0);
date.setYear(50.999999);
assert.sameValue(date.getFullYear(), 1950, 'y = 50.999999');

date = new Date(1970, 0);
date.setYear(99);
assert.sameValue(date.getFullYear(), 1999, 'y = 99');

date = new Date(1970, 0);
date.setYear(99.999999);
assert.sameValue(date.getFullYear(), 1999, 'y = 99.999999');
