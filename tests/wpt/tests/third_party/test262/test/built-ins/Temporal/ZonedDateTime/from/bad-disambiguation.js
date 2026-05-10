// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Throw on bad value for disambiguation
features: [Temporal]
---*/

[
  "",
  "EARLIER",
  "balance"
].forEach(disambiguation => {
  assert.throws(RangeError, () => Temporal.ZonedDateTime.from("2020-11-01T04:00[-08:00]", { disambiguation }));
});
