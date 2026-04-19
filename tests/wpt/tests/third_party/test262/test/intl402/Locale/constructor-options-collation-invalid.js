// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the options argument to the Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    18. If collation is not undefined, then
      a. If collation does not match the [(3*8alphanum) *("-" (3*8alphanum))] sequence, throw a RangeError exception.

features: [Intl.Locale]
---*/


/*
 alphanum = (ALPHA / DIGIT)     ; letters and numbers
 collation = (3*8alphanum) *("-" (3*8alphanum))
*/
const invalidCollationOptions = [
  "",
  "a",
  "ab",
  "abcdefghi",
  "abc-abcdefghi",
];
for (const invalidCollationOption of invalidCollationOptions) {
  assert.throws(RangeError, function() {
    new Intl.Locale("en", {collation: invalidCollationOption});
  });
}
