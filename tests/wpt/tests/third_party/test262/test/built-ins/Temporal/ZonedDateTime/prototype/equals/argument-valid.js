// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Built-in time zones are compared correctly out of valid strings
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const validsEqual = [
  ["+0330", "+03:30"],
  ["-0650", "-06:50"],
  ["-08", "-08:00"],
  ["1994-11-05T08:15:30-05:00", "-05:00"],
  ["1994-11-05T13:15:30Z", "UTC"]
];

for (const [valid, canonical] of validsEqual) {
  assert(instance.withTimeZone(valid).equals(instance.withTimeZone(canonical)), `${valid} time zone equals ${canonical}`);
  assert(instance.withTimeZone(canonical).equals(instance.withTimeZone(valid)), `${canonical} time zone equals ${valid}`);
}

const validsNotEqual = [
  ["+0330", "+03:31"],
  ["-0650", "-06:51"],
  ["-08", "-08:01"],
  ["1994-11-05T08:15:30-05:00", "-05:01"],
];

for (const [valid, canonical] of validsNotEqual) {
  assert(!instance.withTimeZone(valid).equals(instance.withTimeZone(canonical)), `${valid} time zone does not equal ${canonical}`);
  assert(!instance.withTimeZone(canonical).equals(instance.withTimeZone(valid)), `${canonical} time zone does not equal ${valid}`);
}
