// Copyright (C) 2018 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Negative years must be serialized with at least four digits
esid: sec-date.prototype.tostring
info: |
    Date.prototype.toString ( )

    2. Return ToDateString(_tv_).
---*/

var negative1DigitYearToString = (new Date('-000001-07-01T00:00Z')).toString();
var negative2DigitYearToString = (new Date('-000012-07-01T00:00Z')).toString();
var negative3DigitYearToString = (new Date('-000123-07-01T00:00Z')).toString();
var negative4DigitYearToString = (new Date('-001234-07-01T00:00Z')).toString();
var negative5DigitYearToString = (new Date('-012345-07-01T00:00Z')).toString();
var negative6DigitYearToString = (new Date('-123456-07-01T00:00Z')).toString();

// Date.prototype.toString emits values like
// "Fri Mar 28 2019 10:23:42 GMT-0400 (Eastern Daylight Time)".
// Extract and verify just the year.
assert.sameValue(negative1DigitYearToString.split(' ')[3], '-0001',
    'Date.prototype.toString serializes year -1 to "-0001"');
assert.sameValue(negative2DigitYearToString.split(' ')[3], '-0012',
    'Date.prototype.toString serializes year -12 to "-0012"');
assert.sameValue(negative3DigitYearToString.split(' ')[3], '-0123',
    'Date.prototype.toString serializes year -123 to "-0123"');
assert.sameValue(negative4DigitYearToString.split(' ')[3], '-1234',
    'Date.prototype.toString serializes year -1234 to "-1234"');
assert.sameValue(negative5DigitYearToString.split(' ')[3], '-12345',
    'Date.prototype.toString serializes year -12345 to "-12345"');
assert.sameValue(negative6DigitYearToString.split(' ')[3], '-123456',
    'Date.prototype.toString serializes year -123456 to "-123456"');
