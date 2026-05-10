// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: Time separator in string argument can vary
features: [Temporal]
---*/

const tests = [
  ["1970-01-01T00:00Z", "uppercase T"],
  ["1970-01-01t00:00Z", "lowercase T"],
  ["1970-01-01 00:00Z", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.Instant.from(arg);

  assert.sameValue(
    result.epochNanoseconds,
    0n,
    `variant time separators (${description})`
  );
});
