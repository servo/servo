// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: Basic tests for time zone IDs
features: [Temporal, canonical-tz]
---*/

const valid = [
  ["Europe/Vienna"],
  ["America/New_York"],
  ["Africa/CAIRO", "Africa/Cairo"],
  ["africa/cairo", "Africa/Cairo"],
  ["Asia/Ulaanbaatar"],
  ["Asia/Ulan_Bator"],
  ["UTC"],
  ["GMT"]
];
for (const [zone, id = zone] of valid) {
  const result = new Temporal.ZonedDateTime(0n, zone);
  assert.sameValue(typeof result, "object", `object should be created for ${zone}`);
  assert.sameValue(result.timeZoneId, id, `id for ${zone} should be ${id}`);
}

const invalid = ["+00:01.1", "-01.1"];
for (const zone of invalid) {
  assert.throws(RangeError, () => new Temporal.ZonedDateTime(0n, zone), `should throw for ${zone}`);
}
