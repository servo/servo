// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: timeZoneName option affects formatting of the instance's time zone
locale: [en-US]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "Europe/Vienna");

const resultShort = datetime.toLocaleString("en-US", { timeZoneName: "short" });
const resultLong = datetime.toLocaleString("en-US", { timeZoneName: "long" });
assert.notSameValue(resultShort, resultLong, "formats with different timeZoneName options should be different");
assert(resultLong.includes("Central European Standard Time"), "time zone name can be written out in full");
