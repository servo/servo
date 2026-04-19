// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: >
  AdjustDateTimeStyleFormat should not adjust format when no conflicting options
  are present. See https://github.com/tc39/proposal-temporal/issues/3062
features: [Temporal]
locale: [ja]
---*/

const date = new Date(0);
const instant = new Temporal.Instant(0n, "UTC");

const options = { dateStyle: "full" };

assert.sameValue(
  date.toLocaleString("ja", options),
  instant.toLocaleString("ja", options)
);
