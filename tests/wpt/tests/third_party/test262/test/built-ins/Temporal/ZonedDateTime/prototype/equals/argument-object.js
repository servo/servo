// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Tests that objects can be compared for equality
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const idsEqual0000 = [
  "+00:00",
  "+0000",
  "+00"
];

for (const arg of idsEqual0000) {
  for (const receiver of idsEqual0000) {
    const result = instance.withTimeZone(receiver).equals(instance.withTimeZone(arg));
    assert.sameValue(result, true, `Receiver ${receiver} should equal argument ${arg}`);
  }
}
