// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: Tests that offset time zones are correctly normalized in resolvedOptions() output.
---*/
let validOffsetTimeZones = {
    '-00': '+00:00',
    '-00:00': '+00:00',
    '+00': '+00:00',
    '+0000': '+00:00',
    '+0300': '+03:00',
    '+03:00': '+03:00',
    '+13:00': '+13:00',
    '+2300': '+23:00',
    '-07:00': '-07:00',
    '-14': '-14:00',
    '-2100': '-21:00',
    '+0103': '+01:03',
    '+15:59': '+15:59',
    '+2227': '+22:27',
    '-02:32': '-02:32',
    '-1701': '-17:01',
    '-22:23': '-22:23',
};
Object.keys(validOffsetTimeZones).forEach((timeZone) => {
    let df = new Intl.DateTimeFormat(undefined, {timeZone});
    let expected = validOffsetTimeZones[timeZone];
    assert.sameValue(df.resolvedOptions().timeZone, expected, timeZone);
});
