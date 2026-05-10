// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Test Unicode extension subtag canonicalisation for the "sd" extension key.
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

    Replace aliases in special key values:
      If there is an 'sd' or 'rg' key, replace any subdivision alias in its value in the same way,
      using subdivisionAlias data.
includes: [testIntl.js]
---*/

const testData = {
  // <subdivisionAlias type="no23" replacement="no50" reason="deprecated"/>
  "no23": "no50",

  // <subdivisionAlias type="cn11" replacement="cnbj" reason="deprecated"/>
  "cn11": "cnbj",

  // <subdivisionAlias type="cz10a" replacement="cz110" reason="deprecated"/>
  "cz10a": "cz110",

  // <subdivisionAlias type="fra" replacement="frges" reason="deprecated"/>
  "fra": "frges",

  // <subdivisionAlias type="frg" replacement="frges" reason="deprecated"/>
  "frg": "frges",

  // <subdivisionAlias type="lud" replacement="lucl ludi lurd luvd luwi" reason="deprecated"/>
  "lud": "lucl",
};

for (let [alias, name] of Object.entries(testData)) {
  // Subdivision codes should always have a matching region subtag. This
  // shouldn't actually matter for canonicalisation, but let's not push our
  // luck and instead keep the language tag 'valid' per UTS 35, §3.6.5.
  let region = name.substring(0, 2).toUpperCase();

  let tag = `und-${region}-u-sd-${alias}`;
  let canonical = `und-${region}-u-sd-${name}`;

  // Make sure the test data is correct.
  assert.sameValue(isCanonicalizedStructurallyValidLanguageTag(tag), false,
                   "\"" + tag + "\" isn't a canonical language tag.");
  assert(isCanonicalizedStructurallyValidLanguageTag(canonical),
         "\"" + canonical + "\" is a canonical and structurally valid language tag.");

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], canonical);
}
