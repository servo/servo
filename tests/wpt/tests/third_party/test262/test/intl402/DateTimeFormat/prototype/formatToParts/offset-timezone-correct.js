// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  Tests that formatted hour and minute are correct for offset time zones.
---*/
let date = new Date('1995-12-17T03:24:56Z');
let tests = {
    '+0301': {hour: "6", minute: "25"},
    '+1412': {hour: "5", minute: "36"},
    '+02':   {hour: "5", minute: "24"},
    '+13:49': {hour: "5", minute: "13"},
    '-07:56': {hour: "7", minute: "28"},
    '-12': {hour: "3", minute: "24"},
    '-0914': {hour: "6", minute: "10"},
    '-10:03': {hour: "5", minute: "21"},
    '-0509': {hour: "10", minute: "15"},
};
Object.entries(tests).forEach(([timeZone, expected]) => {
    let df = new Intl.DateTimeFormat("en",
        {timeZone, timeStyle: "short"});
    let res = df.formatToParts(date);
    let hour = res.filter((t) => t.type === "hour")[0].value
    let minute = res.filter((t) => t.type === "minute")[0].value
    assert.sameValue(hour, expected.hour, `hour in ${timeZone} time zone:`);
    assert.sameValue(minute, expected.minute, `minute in ${timeZone} time zone:`);
});
