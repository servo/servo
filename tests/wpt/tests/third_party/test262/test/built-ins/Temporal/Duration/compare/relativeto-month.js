// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo with months.
features: [Temporal]
---*/

const oneMonth = new Temporal.Duration(0, 1);
const days30 = new Temporal.Duration(0, 0, 0, 30);
assert.sameValue(
  Temporal.Duration.compare(oneMonth, days30, { relativeTo: Temporal.PlainDate.from("2018-04-01") }), 0);
assert.sameValue(
  Temporal.Duration.compare(oneMonth, days30, { relativeTo: Temporal.PlainDate.from("2018-03-01") }), 1);
assert.sameValue(
  Temporal.Duration.compare(oneMonth, days30, { relativeTo: Temporal.PlainDate.from("2018-02-01") }), -1);
