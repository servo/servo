// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Time separator in string argument can vary
features: [Temporal]
---*/

const dateTime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23);
const tests = [
  ["1976-11-18T15:23", "uppercase T"],
  ["1976-11-18t15:23", "lowercase T"],
  ["1976-11-18 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  assert.sameValue(
    Temporal.PlainDateTime.compare(arg, dateTime),
    0,
    `variant time separators (${description}), first argument`
  );

  assert.sameValue(
    Temporal.PlainDateTime.compare(dateTime, arg),
    0,
    `variant time separators (${description}), second argument`
  );
});
