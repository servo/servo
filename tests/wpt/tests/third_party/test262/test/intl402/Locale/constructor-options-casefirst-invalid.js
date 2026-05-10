// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    22. Let kf be ? GetOption(options, "caseFirst", "string", « "upper", "lower", "false" », undefined).
    ...

    GetOption ( options, property, type, values, fallback )
    ...
    2.  d. If values is not undefined, then
            i. If values does not contain an element equal to value, throw a RangeError exception.
    ...
features: [Intl.Locale]
---*/


const invalidCaseFirstOptions = [
  "",
  "u",
  "Upper",
  "upper\0",
  "uppercase",
  "true",
  { valueOf() { return false; } },
];
for (const caseFirst of invalidCaseFirstOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {caseFirst});
  }, `new Intl.Locale("en", {caseFirst: "${caseFirst}"}) throws RangeError`);
}
