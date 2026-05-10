// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Test Unicode extension subtag canonicalisation for the "ca" extension key.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
    1. Let ll be ? CanonicalizeLocaleList(locales).
    2. Return CreateArrayFromList(ll).

  9.2.1 CanonicalizeLocaleList (locales)
    ...
    7. Repeat, while k < len
      ...
      c. If kPresent is true, then
        ...
        v. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
        vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
        ...

  UTS 35, §3.2.1 Canonical Unicode Locale Identifiers
    Use the bcp47 data to replace keys, types, tfields, and tvalues by their canonical forms.
    See Section 3.6.4 U Extension Data Files) and Section 3.7.1 T Extension Data Files. The
    aliases are in the alias attribute value, while the canonical is in the name attribute value.
includes: [testIntl.js]
---*/

// <key name="ca" [...] alias="calendar">
const testData = {
  // <type name="ethioaa" [...] alias="ethiopic-amete-alem"/>
  "ethiopic-amete-alem": "ethioaa",

  // <type name="islamic-civil" [...] />
  // <type name="islamicc" [...] deprecated="true" preferred="islamic-civil" alias="islamic-civil"/>
  //
  // "name" and "alias" for "islamic-civil" don't quite match of what's spec'ed in UTS 35, §3.2.1.
  // Specifically following §3.2.1 to the letter means "islamicc" is the canonical value whereas
  // "islamic-civil" is an alias value. Assume the definitions in
  // https://unicode.org/reports/tr35/#Unicode_Locale_Extension_Data_Files overrule UTS 35, §3.2.1.
  "islamicc": "islamic-civil",
};

for (let [alias, name] of Object.entries(testData)) {
  let tag = "und-u-ca-" + alias;
  let canonical = "und-u-ca-" + name;

  // Make sure the test data is correct.
  assert.sameValue(isCanonicalizedStructurallyValidLanguageTag(tag), false,
                   "\"" + tag + "\" isn't a canonical language tag.");
  assert(isCanonicalizedStructurallyValidLanguageTag(canonical),
         "\"" + canonical + "\" is a canonical and structurally valid language tag.");

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], canonical);
}
