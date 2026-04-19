// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: >
  GetStartOfDay throws a RangeError for values outside the valid limits.
info: |
  Temporal.ZonedDateTime.prototype.withPlainTime ( [ plainTimeLike ] )

  ...
  6. If plainTimeLike is undefined, then
    a. Let epochNs be ? GetStartOfDay(timeZone, isoDateTime.[[ISODate]]).
  ...
features: [Temporal]
---*/

var zdt;

zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "-01");
assert.throws(RangeError, () => zdt.withPlainTime());

zdt = new Temporal.ZonedDateTime(-864n * 10n**19n, "+01");
assert.throws(RangeError, () => zdt.withPlainTime());
