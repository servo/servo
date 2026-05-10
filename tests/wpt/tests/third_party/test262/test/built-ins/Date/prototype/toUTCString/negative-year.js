// Copyright (C) 2018 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Negative years must be serialized with at least four digits
esid: sec-date.prototype.toutcstring
info: |
    Date.prototype.toUTCString ( )

    10. Return the string-concatenation of _weekday_, `","`, the code unit 0x0020 (SPACE), _day_, the code unit 0x0020 (SPACE), _month_, the code unit 0x0020 (SPACE), _yearSign_, _year_, the code unit 0x0020 (SPACE), and TimeString(_tv_).
---*/

var negative1DigitYearToUTCString = (new Date('-000001-07-01T00:00Z')).toUTCString();
var negative2DigitYearToUTCString = (new Date('-000012-07-01T00:00Z')).toUTCString();
var negative3DigitYearToUTCString = (new Date('-000123-07-01T00:00Z')).toUTCString();
var negative4DigitYearToUTCString = (new Date('-001234-07-01T00:00Z')).toUTCString();
var negative5DigitYearToUTCString = (new Date('-012345-07-01T00:00Z')).toUTCString();
var negative6DigitYearToUTCString = (new Date('-123456-07-01T00:00Z')).toUTCString();

// Date.prototype.toUTCString emits values like "Thu, 28 Mar 2019 10:23:42 GMT".
// Extract and verify just the year.
assert.sameValue(negative1DigitYearToUTCString.split(' ')[3], '-0001',
    'Date.prototype.toUTCString serializes year -1 to "-0001"');
assert.sameValue(negative2DigitYearToUTCString.split(' ')[3], '-0012',
    'Date.prototype.toUTCString serializes year -12 to "-0012"');
assert.sameValue(negative3DigitYearToUTCString.split(' ')[3], '-0123',
    'Date.prototype.toUTCString serializes year -123 to "-0123"');
assert.sameValue(negative4DigitYearToUTCString.split(' ')[3], '-1234',
    'Date.prototype.toUTCString serializes year -1234 to "-1234"');
assert.sameValue(negative5DigitYearToUTCString.split(' ')[3], '-12345',
    'Date.prototype.toUTCString serializes year -12345 to "-12345"');
assert.sameValue(negative6DigitYearToUTCString.split(' ')[3], '-123456',
    'Date.prototype.toUTCString serializes year -123456 to "-123456"');
