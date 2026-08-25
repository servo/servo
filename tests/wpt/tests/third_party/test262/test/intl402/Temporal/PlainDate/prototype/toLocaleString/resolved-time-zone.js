// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2021, 8, 4);
const result = date.toLocaleString("en", { timeZone: "Pacific/Apia" });
assert.sameValue(result, "8/4/2021");
