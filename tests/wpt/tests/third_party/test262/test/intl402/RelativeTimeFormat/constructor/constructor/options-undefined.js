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
  style: {
    get() {
      throw new Error("Should not call style getter");
    }
  },
  numeric: {
    get() {
      throw new Error("Should not call numeric getter");
    }
  },
})

const optionsArguments = [
  [],
  [[]],
  [[], undefined],
];

for (const args of optionsArguments) {
  const rtf = new Intl.RelativeTimeFormat(...args);
  const resolvedOptions = rtf.resolvedOptions();
  assert.sameValue(resolvedOptions.style, "long",
    `Calling with ${args.length} empty arguments should yield the fallback value for "style"`);
  assert.sameValue(resolvedOptions.numeric, "always",
    `Calling with ${args.length} empty arguments should yield the fallback value for "numeric"`);
}
