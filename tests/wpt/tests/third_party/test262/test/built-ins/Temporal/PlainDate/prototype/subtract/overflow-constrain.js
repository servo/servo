// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Constrains with overflow constrain
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const mar31 = Temporal.PlainDate.from("2020-03-31");
TemporalHelpers.assertPlainDate(mar31.subtract({ months: 1 }, { overflow: "constrain" }),
  2020, 2, "M02", 29);
