// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Various forms of unknown annotation
features: [Temporal]
---*/

const tests = [
  ["1970-01-01T00:00[UTC][foo=bar]", "with time zone"],
  ["1970-01-01T00:00[UTC][foo=bar][u-ca=iso8601]", "before calendar"],
  ["1970-01-01T00:00[UTC][u-ca=iso8601][foo=bar]", "after calendar"],
  ["1970-01-01T00:00[UTC][foo=bar][_foo-bar0=Ignore-This-999999999999]", "with another unknown annotation"],
];

tests.forEach(([arg, description]) => {
  const result = Temporal.ZonedDateTime.compare(arg, arg);

  assert.sameValue(
    result,
    0,
    `unknown annotation (${description})`
  );
});
