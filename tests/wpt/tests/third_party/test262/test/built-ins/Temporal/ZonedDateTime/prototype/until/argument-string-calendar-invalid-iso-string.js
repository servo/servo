// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Invalid ISO string as calendar should throw RangeError
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

const invalidStrings = [
  ["", "empty string"],
  ["1997-12-04[u-ca=notacal]", "Unknown calendar"],
  ["1997-12-04[u-ca=11111111]", "compact ISO date used as calendar name"],
  ["1997-12-04[u-ca=1111-11-11]", "extended ISO date used as calendar name"],
];

for (const [arg, description] of invalidStrings) {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `${description} is not a valid calendar ID`
  );
}
