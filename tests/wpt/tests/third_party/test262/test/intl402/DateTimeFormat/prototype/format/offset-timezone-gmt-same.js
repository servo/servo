// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  Tests that date and time formatting in an offset time zone
  matches that in the equivalent Etc/GMT±n time zone.
---*/
let offsetTimeZones = {
    '+0300': 'Etc/GMT-3',
    '+1400': 'Etc/GMT-14',
    '+02': 'Etc/GMT-2',
    '+13:00': 'Etc/GMT-13',
    '-07:00': 'Etc/GMT+7',
    '-12': 'Etc/GMT+12',
    '-0900': 'Etc/GMT+9',
};
let date = new Date('1995-12-17T03:24:56Z');
Object.entries(offsetTimeZones).forEach(([offsetZone, gmtZone]) => {
    let offsetDf = new Intl.DateTimeFormat("en",
        {timeZone: offsetZone, dateStyle: "short", timeStyle: "short"});
    let gmtDf = new Intl.DateTimeFormat("en",
        {timeZone: gmtZone, dateStyle: "short", timeStyle: "short"});
    assert.sameValue(offsetDf.format(date), gmtDf.format(date), `${offsetZone} vs. ${gmtZone}:`);
});
