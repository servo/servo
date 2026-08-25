// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tojson
description: Basic behavior for toJSON
features: [BigInt, Temporal]
---*/

const tests = [
  [new Temporal.ZonedDateTime(192_258_181_000_000_000n, "UTC"), "1976-02-04T05:03:01+00:00[UTC]"],
  [new Temporal.ZonedDateTime(0n, "UTC"), "1970-01-01T00:00:00+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_000_000_000n, "UTC"), "1970-01-01T00:00:30+00:00[UTC]"],
  [new Temporal.ZonedDateTime(30_123_400_000n, "UTC"), "1970-01-01T00:00:30.1234+00:00[UTC]"],
];

const options = new Proxy({}, {
  get() { throw new Test262Error("should not get properties off argument") }
});
for (const [datetime, expected] of tests) {
  assert.sameValue(datetime.toJSON(), expected, "toJSON without argument");
  assert.sameValue(datetime.toJSON(options), expected, "toJSON with argument");
}
