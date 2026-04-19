// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks error cases for the string conversion of the locale argument to the
    Locale constructor.
info: |
    Intl.Locale( tag [, options] )

    ...
    8. If Type(tag) is Object and tag has an [[InitializedLocale]] internal slot, then
    9. Else,
        a. Let tag be ? ToString(tag).
features: [Intl.Locale]
---*/

function CustomError() {}
function WrongCustomError() {}

const errors = [
  { get [Symbol.toPrimitive]() { throw new CustomError(); } },
  { [Symbol.toPrimitive](hint) { assert.sameValue(hint, "string"); throw new CustomError(); } },
  { get toString() { throw new CustomError(); }, get valueOf() { throw new WrongCustomError(); } },
  { toString() { throw new CustomError(); }, get valueOf() { throw new WrongCustomError(); } },
  { toString: undefined, get valueOf() { throw new CustomError(); } },
  { toString: undefined, valueOf() { throw new CustomError(); } },
  { toString() { return {} }, get valueOf() { throw new CustomError(); } },
  { toString() { return {} }, valueOf() { throw new CustomError(); } },
];

for (const input of errors) {
  assert.throws(CustomError, function() {
    new Intl.Locale(input);
  });
}
