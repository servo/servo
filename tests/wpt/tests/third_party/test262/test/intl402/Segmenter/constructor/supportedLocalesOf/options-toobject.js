// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.supportedLocalesOf
description: Checks handling of non-object options arguments to the supportedLocalesOf function.
info: |
    SupportedLocales ( availableLocales, requestedLocales, options )

    1. If options is not undefined, then
        a. Let options be ? ToObject(options).
features: [Intl.Segmenter]
---*/

assert.sameValue(typeof Intl.Segmenter.supportedLocalesOf, "function",
                 "Should support Intl.Segmenter.supportedLocalesOf.");

let called;
Object.defineProperties(Object.prototype, {
  "localeMatcher": {
    get() {
      ++called;
      return "best fit";
    }
  }
});

const optionsArguments = [
  true,
  "test",
  7,
  Symbol(),
];

for (const options of optionsArguments) {
  called = 0;
  const result = Intl.Segmenter.supportedLocalesOf([], options);
  assert.sameValue(Array.isArray(result), true, `Expected array from ${String(options)}`);
  assert.sameValue(called, 1, `Expected one call from ${String(options)}`);
}

