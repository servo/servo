// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.supportedLocalesOf
description: Checks handling of invalid values for the localeMatcher option to the supportedLocalesOf function.
info: |
    SupportedLocales ( availableLocales, requestedLocales, options )

    1. If options is not undefined, then
        b. Let matcher be ? GetOption(options, "localeMatcher", "string", «"lookup", "best fit"», "best fit").
features: [Intl.ListFormat]
---*/

assert.sameValue(typeof Intl.ListFormat.supportedLocalesOf, "function",
                 "Should support Intl.ListFormat.supportedLocalesOf.");

const invalidOptions = [
  null,
  1,
  "",
  "Lookup",
  "LOOKUP",
  "lookup\0",
  "Best fit",
  "BEST FIT",
  "best\u00a0fit",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    Intl.ListFormat.supportedLocalesOf([], {"localeMatcher": invalidOption});
  }, `${invalidOption} is an invalid localeMatcher option value`);
}
