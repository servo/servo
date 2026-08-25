// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Hour cycle should be correctly set when defaults are used
features: [Temporal]
---*/

const output0 = "00:00:00";
const output11 = "0:00:00";
const output12 = "12:00:00";
const output24 = "24:00:00";

const item = new Temporal.PlainTime(0, 0);
var result = item.toLocaleString("en", { hour12: false, timeZone: "UTC" });
assert.sameValue(result.includes(output0), true, `output for hour12: false should include ${output0}`);

result = item.toLocaleString("en", { hour12: true, timeZone: "UTC" });
assert.sameValue(result.includes(output12), true, `output for hour12: true should include ${output12}`);

result = item.toLocaleString("en", { hourCycle: "h23", timeZone: "UTC" });
assert.sameValue(result.includes(output0), true, `output for hourCycle: h23 should include ${output0}`);

result = item.toLocaleString("en", { hourCycle: "h24", timeZone: "UTC" });
assert.sameValue(result.includes(output24), true, `output for hourCycle: h24 should include ${output24}`);

result = item.toLocaleString("en", { hourCycle: "h11", timeZone: "UTC" });
assert.sameValue(result.includes(output11), true, `output for hourCycle: h11 should include ${output11}`);

result = item.toLocaleString("en", { hourCycle: "h12", timeZone: "UTC" });
assert.sameValue(result.includes(output12), true, `output for hourCycle: h12 should include ${output12}`);
