// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Time separator in string argument can vary
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const tests = [
  ["2000-05-02T15:23", "uppercase T"],
  ["2000-05-02t15:23", "lowercase T"],
  ["2000-05-02 15:23", "space between date and time"],
];

tests.forEach(([arg, description]) => {
  assert.sameValue(
    Temporal.PlainDate.compare(arg, date),
    0,
    `variant time separators (${description}), first argument`
  );

  assert.sameValue(
    Temporal.PlainDate.compare(date, arg),
    0,
    `variant time separators (${description}), second argument`
  );
});
