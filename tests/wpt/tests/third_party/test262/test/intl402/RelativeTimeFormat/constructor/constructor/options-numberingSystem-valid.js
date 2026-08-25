// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks valid cases for the options argument to the RelativeTimeFormat constructor.
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat, "function");

const validNumberingSystemOptions = [
  "abc",
  "abcd",
  "abcde",
  "abcdef",
  "abcdefg",
  "abcdefgh",
  "12345678",
  "1234abcd",
  "1234abcd-abc123",
];

for (const numberingSystem of validNumberingSystemOptions) {
  const rtf = new Intl.RelativeTimeFormat("en", {numberingSystem});
  assert.sameValue(rtf.resolvedOptions().numberingSystem, "latn");
}
