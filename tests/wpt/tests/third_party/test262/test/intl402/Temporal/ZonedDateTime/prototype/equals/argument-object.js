// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Objects with IANA IDs are compared case-insensitively with their canonical IDs
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const namesIANA = [
  "Asia/Calcutta",
  "Asia/Kolkata",
  "ASIA/calcutta",
  "Asia/KOLKATA",
];

for (const id1 of namesIANA) {
  for (const id2 of namesIANA) {
    assert(
      instance.withTimeZone(id1).equals(instance.withTimeZone(id2)),
      `Receiver ${id1} should equal argument ${id2}`
    );
  }
}

const namesIANADifferentCanonical = [
  "Asia/Colombo",
  "ASIA/colombo",
];

for (const id1 of namesIANADifferentCanonical) {
  for (const id2 of namesIANA) {
    assert(
      !instance.withTimeZone(id1).equals(instance.withTimeZone(id2)),
      `Receiver ${id1} should not equal argument ${id2}`
    );
    assert(
      !instance.withTimeZone(id2).equals(instance.withTimeZone(id1)),
      `Receiver ${id2} should not equal argument ${id1}`
    );
  }
}
