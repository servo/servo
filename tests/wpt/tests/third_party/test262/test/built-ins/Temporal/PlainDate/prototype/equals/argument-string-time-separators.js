// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: Time separator in string argument can vary
features: [Temporal]
---*/

const tests = [
  ["2000-05-02T15:23", "uppercase T"],
  ["2000-05-02t15:23", "lowercase T"],
  ["2000-05-02 15:23", "space between date and time"],
];

const instance = new Temporal.PlainDate(2000, 5, 2);

tests.forEach(([arg, description]) => {
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `variant time separators (${description})`
  );
});
