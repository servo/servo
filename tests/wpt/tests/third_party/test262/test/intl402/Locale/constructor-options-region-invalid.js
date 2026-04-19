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
    8. If region is not undefined, then
        a. If region does not match the region production, throw a RangeError exception.
    ...

features: [Intl.Locale]
---*/

/*
 region        = 2ALPHA              ; ISO 3166-1 code
               / 3DIGIT              ; UN M.49 code
*/
const invalidRegionOptions = [
  "",
  "a",
  "abc",
  "a7",

  // Value cannot be parsed as a 'region' production.
  "notaregion",

  // Value contains more than just the 'region' production.
  "SA-vaidika",
  "SA-a-asdf",
  "SA-x-private",

  // Value contains more than just the 'script' production.
  "ary-Arab",
  "Latn-SA",
  "Latn-vaidika",
  "Latn-a-asdf",
  "Latn-x-private",

  7,
];
for (const region of invalidRegionOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {region});
  }, `new Intl.Locale("en", {region: "${region}"}) throws RangeError`);
}
