// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo with years.
features: [Temporal]
---*/

const oneYear = new Temporal.Duration(1);
const days365 = new Temporal.Duration(0, 0, 0, 365);
assert.sameValue(
  Temporal.Duration.compare(oneYear, days365, { relativeTo: Temporal.PlainDate.from("2017-01-01") }), 0);
assert.sameValue(
  Temporal.Duration.compare(oneYear, days365, { relativeTo: Temporal.PlainDate.from("2016-01-01") }), 1);
