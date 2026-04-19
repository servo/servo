// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Time separator in string argument can vary
features: [Temporal]
---*/

const epoch = new Temporal.ZonedDateTime(0n, "UTC");
const tests = [
  ["1970-01-01T00:00+00:00[UTC]", "uppercase T"],
  ["1970-01-01t00:00+00:00[UTC]", "lowercase T"],
  ["1970-01-01 00:00+00:00[UTC]", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  assert.sameValue(
    Temporal.ZonedDateTime.compare(arg, epoch),
    0,
    `variant time separators (${description}), first argument`
  );

  assert.sameValue(
    Temporal.ZonedDateTime.compare(epoch, arg),
    0,
    `variant time separators (${description}), second argument`
  );
});
