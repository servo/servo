// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of non-object option arguments to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
features: [Intl.ListFormat]
---*/

Object.defineProperties(Object.prototype, {
  "type": {
    get() {
      throw new Error("Should not call type getter");
    }
  },
  "style": {
    get() {
      throw new Error("Should not call style getter");
    }
  },
})

const optionsArguments = [
  [],
  [[]],
  [[], undefined],
];

for (const args of optionsArguments) {
  const lf = new Intl.ListFormat(...args);
  const resolvedOptions = lf.resolvedOptions();
  assert.sameValue(resolvedOptions.type, "conjunction",
    `Calling with ${args.length} empty arguments should yield the correct value for "type"`);
  assert.sameValue(resolvedOptions.style, "long",
    `Calling with ${args.length} empty arguments should yield the correct value for "style"`);
}
