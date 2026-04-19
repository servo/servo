// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: Tests that offset time zones are correctly normalized in resolvedOptions() output.
---*/
let validOffsetTimeZones = [
    '+03',
    '+13',
    '+23',
    '-07',
    '-14',
    '-21',
    '+01:03',
    '+15:59',
    '+22:27',
    '-02:32',
    '-17:01',
    '-22:23',
];
validOffsetTimeZones.forEach((timeZone) => {
    let df = new Intl.DateTimeFormat(undefined, {timeZone});
    let expected = timeZone;
    if (expected.length == 3) {
        expected += ":00";
    }
    assert.sameValue(df.resolvedOptions().timeZone, expected, timeZone);
});
