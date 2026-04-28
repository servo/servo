// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of non-object option arguments to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
features: [Intl.ListFormat,BigInt]
---*/

const optionsArguments = [
  null,
  true,
  false,
  "test",
  7,
  Symbol(),
  123456789n,
];

for (const options of optionsArguments) {
  assert.throws(TypeError, function() { new Intl.ListFormat([], options) })
}
