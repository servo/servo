// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Invalid ISO string not acceptable even with overflow = constrain
features: [Temporal]
---*/

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("2020-13-34T24:60", {}),
  "constrain has no effect on invalid ISO string (empty options argument)"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("2020-13-34T24:60", () => {}),
  "constrain has no effect on invalid ISO string (nullary empty object function argument)"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("2020-13-34T24:60", {overflow: "constrain"}),
  "overflow = constrain has no effect on invalid ISO string"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from("2020-13-34T24:60", {overflow: "reject"}),
  "overflow = reject has no effect on invalid ISO string"
);
