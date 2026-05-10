// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: >
    Behavior when the integer representation of the specified `year` is not
    relative to 1900
info: |
    [...]
    5. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yyyy be ToInteger(y) +
       1900.
    6. Else, let yyyy be y.
    [...]
---*/

var date;

date = new Date(1970, 0);
date.setYear(-1);
assert.sameValue(date.getFullYear(), -1);

date = new Date(1970, 0);
date.setYear(100);
assert.sameValue(date.getFullYear(), 100);

date = new Date(1970, 0);
date.setYear(1899);
assert.sameValue(date.getFullYear(), 1899);

date = new Date(1970, 0);
date.setYear(1900);
assert.sameValue(date.getFullYear(), 1900);

date = new Date(1970, 0);
date.setYear(1999);
assert.sameValue(date.getFullYear(), 1999);

date = new Date(1970, 0);
date.setYear(2000);
assert.sameValue(date.getFullYear(), 2000);
