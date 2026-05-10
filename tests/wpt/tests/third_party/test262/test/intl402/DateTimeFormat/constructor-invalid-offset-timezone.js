// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: Tests that invalid offset time zones are rejected.
---*/
let invalidOffsetTimeZones = [
    '+3',
    '+24',
    '+23:0',
    '+130',
    '+13234',
    '+135678',
    '-7',
    '-10.50',
    '-10,50',
    '-24',
    '-014',
    '-210',
    '-2400',
    '-1:10',
    '-21:0',
    '+0:003',
    '+15:59:00',
    '+15:59.50',
    '+15:59,50',
    '+222700',
    '-02:3200',
    '-170100',
    '-22230',
];
invalidOffsetTimeZones.forEach((timeZone) => {
    assert.throws(RangeError, function() {
        new Intl.DateTimeFormat(undefined, {timeZone});
    }, timeZone + ":");
});
