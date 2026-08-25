// Copyright (C) 2018 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Negative years must be serialized with at least four digits
esid: sec-date.prototype.todatestring
info: |
    Date.prototype.toDateString ( )

    5. Return DateString(_t_).
---*/

var negative1DigitYearDateString = (new Date('-000001-07-01T00:00Z')).toDateString();
var negative2DigitYearDateString = (new Date('-000012-07-01T00:00Z')).toDateString();
var negative3DigitYearDateString = (new Date('-000123-07-01T00:00Z')).toDateString();
var negative4DigitYearDateString = (new Date('-001234-07-01T00:00Z')).toDateString();
var negative5DigitYearDateString = (new Date('-012345-07-01T00:00Z')).toDateString();
var negative6DigitYearDateString = (new Date('-123456-07-01T00:00Z')).toDateString();

// Date.prototype.toDateString emits values like "Fri Mar 28 2019".
// Extract and verify just the year.
assert.sameValue(negative1DigitYearDateString.split(' ')[3], '-0001',
    'Date.prototype.toDateString serializes year -1 to "-0001"');
assert.sameValue(negative2DigitYearDateString.split(' ')[3], '-0012',
    'Date.prototype.toDateString serializes year -12 to "-0012"');
assert.sameValue(negative3DigitYearDateString.split(' ')[3], '-0123',
    'Date.prototype.toDateString serializes year -123 to "-0123"');
assert.sameValue(negative4DigitYearDateString.split(' ')[3], '-1234',
    'Date.prototype.toDateString serializes year -1234 to "-1234"');
assert.sameValue(negative5DigitYearDateString.split(' ')[3], '-12345',
    'Date.prototype.toDateString serializes year -12345 to "-12345"');
assert.sameValue(negative6DigitYearDateString.split(' ')[3], '-123456',
    'Date.prototype.toDateString serializes year -123456 to "-123456"');
