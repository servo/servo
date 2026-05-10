// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Test Unicode extension subtag canonicalisation for the "tz" extension key.
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

// <key name="tz" [...] alias="timezone">
const testData = {
  // Similar to the "ca" extension key, assume "preferred" holds the canonical
  // value and "name" the alias value.

  // <type name="cnckg" [...] deprecated="true" preferred="cnsha"/>
  "cnckg": "cnsha",

  // NB: "Eire" matches the |uvalue| production.
  // <type name="iedub" [...] alias="Europe/Dublin Eire"/>
  "eire": "iedub",

  // NB: "EST" matches the |uvalue| production.
  // <type name="papty" [...] alias="America/Panama EST"/>
  "est": "papty",

  // NB: "GMT0" matches the |uvalue| production.
  // <type name="gmt" [...] alias="Etc/GMT Etc/GMT+0 Etc/GMT-0 Etc/GMT0 Etc/Greenwich GMT GMT+0 GMT-0 GMT0 Greenwich"/>
  "gmt0": "gmt",

  // NB: "UCT" matches the |uvalue| production.
  // <type name="utc" [...] alias="Etc/UTC Etc/UCT Etc/Universal Etc/Zulu UCT UTC Universal Zulu"/>
  "uct": "utc",

  // NB: "Zulu" matches the |uvalue| production.
  // <type name="utc" [...] alias="Etc/UTC Etc/UCT Etc/Universal Etc/Zulu UCT UTC Universal Zulu"/>
  "zulu": "utc",
};

for (let [alias, name] of Object.entries(testData)) {
  let tag = "und-u-tz-" + alias;
  let canonical = "und-u-tz-" + name;

  // Make sure the test data is correct.
  assert.sameValue(isCanonicalizedStructurallyValidLanguageTag(tag), false,
                   "\"" + tag + "\" isn't a canonical language tag.");
  assert(isCanonicalizedStructurallyValidLanguageTag(canonical),
         "\"" + canonical + "\" is a canonical and structurally valid language tag.");

  let result = Intl.getCanonicalLocales(tag);
  assert.sameValue(result.length, 1);
  assert.sameValue(result[0], canonical);
}
