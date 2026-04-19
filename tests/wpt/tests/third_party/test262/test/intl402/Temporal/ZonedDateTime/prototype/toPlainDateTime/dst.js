// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindatetime
description: Sample of results for IANA time zones around DST changes
includes: [temporalHelpers.js]
features: [Temporal]
---*/

function test(epochNs, id, expected) {
  const instance = new Temporal.ZonedDateTime(epochNs, id);
  const dt = instance.toPlainDateTime();
  TemporalHelpers.assertPlainDateTime(dt, ...expected, `Local time of ${instance.toInstant()} in ${id}`);
}

// Just before DST forward shift
test(1553993999_999_999_999n, "Europe/London", [2019, 3, "M03", 31, 0, 59, 59, 999, 999, 999]);
// Just after DST forward shift
test(1553994000_000_000_000n, "Europe/London", [2019, 3, "M03", 31, 2, 0, 0, 0, 0, 0]);
// Just before DST backward shift
test(1550368799_999_999_999n, "America/Sao_Paulo", [2019, 2, "M02", 16, 23, 59, 59, 999, 999, 999]);
// Just after DST backward shift
test(1550368800_000_000_000n, "America/Sao_Paulo", [2019, 2, "M02", 16, 23, 0, 0, 0, 0, 0]);
