// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: >
    Checks error cases for the options argument to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)

    ...
    8. If numberingSystem is not undefined, then
        a. If numberingSystem does not match the type sequence (from UTS 35 Unicode Locale Identifier, section 3.2), throw a RangeError exception.

features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat, "function");

/*
 alphanum = (ALPHA / DIGIT)     ; letters and numbers
 numberingSystem = (3*8alphanum) *("-" (3*8alphanum))
*/
const invalidNumberingSystemOptions = [
  "",
  "a",
  "ab",
  "abcdefghi",
  "abc-abcdefghi",
  "!invalid!",
  "-latn-",
  "latn-",
  "latn--",
  "latn-ca",
  "latn-ca-",
  "latn-ca-gregory",
  "latné",
  "latn编号",
];
for (const numberingSystem of invalidNumberingSystemOptions) {
  assert.throws(RangeError, function() {
    new Intl.RelativeTimeFormat('en', {numberingSystem});
  }, `new Intl.RelativeTimeFormat("en", {numberingSystem: "${numberingSystem}"}) throws RangeError`);
}
