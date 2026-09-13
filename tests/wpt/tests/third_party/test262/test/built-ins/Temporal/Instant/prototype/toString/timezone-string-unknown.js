// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Unrecognized time zone identifiers are not valid input for a time zone
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);

const unknown = ["Mars/Olympus_Mons", "America/Nonexistent"];

for (const timeZone of unknown) {
  assert.throws(
    RangeError,
    () => instance.toString({ timeZone }),
    `${timeZone} is not an available named time zone`
  );

  assert.throws(
    RangeError,
    () => instance.toString({ timeZone: `1970-01-01T00:00+01:00[${timeZone}]` }),
    `${timeZone} in a time zone annotation is not an available named time zone, and the offset is not a fallback`
  );
}
