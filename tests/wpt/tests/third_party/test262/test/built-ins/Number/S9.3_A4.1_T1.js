// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of number conversion from number value equals to the input
    argument (no conversion)
es5id: 9.3_A4.1_T1
description: >
    Some numbers including Number.MAX_VALUE and Number.MIN_VALUE are
    converted to Number with explicit transformation
---*/
assert.sameValue(Number(13), 13, 'Number(13) must return 13');
assert.sameValue(Number(-13), -13, 'Number(-13) must return -13');
assert.sameValue(Number(1.3), 1.3, 'Number(1.3) must return 1.3');
assert.sameValue(Number(-1.3), -1.3, 'Number(-1.3) must return -1.3');
