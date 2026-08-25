// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of non-object option arguments to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
features: [Intl.RelativeTimeFormat]
---*/

Object.defineProperties(Object.prototype, {
  "style": {
    value: "short",
  },
  "numeric": {
    value: "auto",
  },
})

const optionsArguments = [
  true,
  "test",
  7,
  Symbol(),
];

for (const options of optionsArguments) {
  const rtf = new Intl.RelativeTimeFormat([], options);
  const resolvedOptions = rtf.resolvedOptions();
  assert.sameValue(resolvedOptions.style, "short",
    `options argument ${String(options)} should yield the correct value for "style"`);
  assert.sameValue(resolvedOptions.numeric, "auto",
    `options argument ${String(options)} should yield the correct value for "numeric"`);
}
