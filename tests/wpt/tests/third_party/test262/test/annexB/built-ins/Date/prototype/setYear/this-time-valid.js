// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setyear
es6id: B.2.4.2
es5id: B.2.5
description: >
    Behavior when the [[DateValue]] internal slot of "this" value is an integer
    value
info: |
    1. Let t be ? thisTimeValue(this value).
    2. If t is NaN, let t be +0; otherwise, let t be LocalTime(t).
---*/

var date = new Date(1970, 1, 2, 3, 4, 5);
var expected = new Date(1971, 1, 2, 3, 4, 5).valueOf();

assert.sameValue(date.setYear(71), expected, 'method return value');
assert.sameValue(date.valueOf(), expected, '[[DateValue]] internal slot');
