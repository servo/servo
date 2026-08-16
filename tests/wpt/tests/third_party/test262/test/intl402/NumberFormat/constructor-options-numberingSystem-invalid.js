// Copyright 2020 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
    Checks error cases for the options argument to the NumberFormat constructor.
info: |
  ResolveOptions ( constructor, localeData, locales, options [ , specialBehaviours [ , modifyResolutionOptions ] ] )

  1. If _value_ is not *undefined*, then
    1. ...
    1. If _value_ cannot be matched by the `type` Unicode locale nonterminal,
       throw a *RangeError* exception.
---*/

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
    new Intl.NumberFormat('en', {numberingSystem});
  }, `new Intl.NumberFormat("en", {numberingSystem: "${numberingSystem}"}) throws RangeError`);
}
