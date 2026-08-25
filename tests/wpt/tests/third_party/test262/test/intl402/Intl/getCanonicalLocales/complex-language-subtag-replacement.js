// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Assert non-simple language subtag replacements work as expected.
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
        vi. Let canonicalizedTag be CanonicalizeUnicodeLocaleId(tag).
        ...

  UTS 35, §3.2.1 Canonical Unicode Locale Identifiers

  - Replace aliases in the unicode_language_id and tlang (if any) using the following process:
    - If the language subtag matches the type attribute of a languageAlias element in
      Supplemental Data, replace the language subtag with the replacement value.
      1. If there are additional subtags in the replacement value, add them to the result,
         but only if there is no corresponding subtag already in the tag.

includes: [testIntl.js]
---*/

// CLDR contains language mappings where in addition to the language subtag also
// the script or region subtag is modified, unless they're already present.

const testData = {
  // "sh" adds "Latn", unless a script subtag is already present.
  // <languageAlias type="sh" replacement="sr_Latn" reason="legacy"/>
  "sh": "sr-Latn",
  "sh-Cyrl": "sr-Cyrl",

  // "cnr" adds "ME", unless a region subtag is already present.
  // <languageAlias type="cnr" replacement="sr_ME" reason="legacy"/>
  "cnr": "sr-ME",
  "cnr-BA": "sr-BA",
};

for (let [tag, canonical] of Object.entries(testData)) {
  // Make sure the test data is correct.
  assert(
    isCanonicalizedStructurallyValidLanguageTag(canonical),
    "\"" + canonical + "\" is a canonicalized and structurally valid language tag."
  );

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], canonical);
}
