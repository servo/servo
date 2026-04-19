// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
  Verifies the handling of options with grandfathered tags.
info: |
  Intl.Locale( tag [, options] )
  12. Set tag to ? ApplyOptionsToTag(tag, options).
  14. Let calendar be ? GetOption(options, "calendar", "string", undefined, undefined).
  16. Set opt.[[ca]] to calendar.
  30. Let r be ! ApplyUnicodeExtensionToTag(tag, opt, relevantExtensionKeys).

  ApplyOptionsToTag( tag, options )
  ...
  2. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.

  IsStructurallyValidLanguageTag ( locale )

  The IsStructurallyValidLanguageTag abstract operation verifies that the
  locale argument (which must be a String value)

  represents a well-formed Unicode BCP 47 Locale Identifier" as specified in
  Unicode Technical Standard 35 section 3.2, or successor,

features: [Intl.Locale]
---*/

const testData = [
  // Canonicalized version of "en-GB-oed", which we can add "US" to right away.
  {
    tag: "en-GB-oxendict",
    options: {
      region: "US",
      calendar: "gregory",
    },
    canonical: "en-US-oxendict-u-ca-gregory",
  },
];

for (const {tag, options, canonical} of testData) {
  assert.sameValue(
    new Intl.Locale(tag, options).toString(),
    canonical,
    `new Intl.Locale("${tag}", ${options}).toString() returns "${canonical}"`
  );
}

assert.throws(RangeError, () =>
    new Intl.Locale("no-bok", {region: "NO", calendar: "gregory"}));
assert.throws(RangeError, () =>
    new Intl.Locale("no-bok", {region: "SE", calendar: "gregory"}));
assert.throws(RangeError, () =>
    new Intl.Locale("no-bok-NO", {region: "SE", calendar: "gregory"}));
assert.throws(RangeError, () =>
    new Intl.Locale("no-bok-SE", {region: "NO", calendar: "gregory"}));

