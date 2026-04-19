// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale
    constructor.
info: |
    Intl.Locale( tag [, options] )
    10. If options is undefined, then
    11. Else
        a. Let options be ? ToObject(options).
    12. Set tag to ? ApplyOptionsToTag(tag, options).

    ApplyOptionsToTag( tag, options )
    ...
    6. If script is not undefined, then
        a. If script does not match the script production, throw a RangeError exception.
    ...

features: [Intl.Locale]
---*/

/*
 script        = 4ALPHA              ; ISO 15924 code
*/
const invalidScriptOptions = [
  "",
  "a",
  "ab",
  "abc",
  "abc7",
  "notascript",
  "undefined",
  "Bal\u0130",
  "Bal\u0131",

  // Value contains more than just the 'script' production.
  "ary-Arab",
  "Latn-SA",
  "Latn-vaidika",
  "Latn-a-asdf",
  "Latn-x-private",

  7,
];
for (const script of invalidScriptOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {script});
  }, `new Intl.Locale("en", {script: "${script}"}) throws RangeError`);
}
