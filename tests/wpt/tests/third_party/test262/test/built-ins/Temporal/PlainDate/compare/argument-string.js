// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: basic string coercion in arguments
features: [Temporal]
---*/

const d1 = Temporal.PlainDate.from("1976-11-18");
const d2 = Temporal.PlainDate.from("2019-06-30");

assert.sameValue(Temporal.PlainDate.compare("1976-11-18", d2), -1, "first argument");
assert.sameValue(Temporal.PlainDate.compare("2019-06-30", d2), 0, "first argument");
assert.sameValue(Temporal.PlainDate.compare("2024-01-12", d2), 1, "first argument");

assert.sameValue(Temporal.PlainDate.compare(d1, "2019-06-30"), -1, "second argument");
assert.sameValue(Temporal.PlainDate.compare(d1, "1976-11-18"), 0, "second argument");
assert.sameValue(Temporal.PlainDate.compare(d1, "1926-07-07"), 1, "second argument");
