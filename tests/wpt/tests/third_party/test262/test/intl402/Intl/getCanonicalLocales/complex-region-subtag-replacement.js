// Copyright (C) 2020 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Assert non-simple region subtag replacements work as expected.
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
    - If the region subtag matches the type attribute of a territoryAlias element in
      Supplemental Data, replace the language subtag with the replacement value, as follows:
      1. If there is a single territory in the replacement, use it.
      2. If there are multiple territories:
        1. Look up the most likely territory for the base language code (and script, if there is one).
        2. If that likely territory is in the list, use it.
        3. Otherwise, use the first territory in the list.

includes: [testIntl.js]
---*/

// CLDR contains region mappings where the replacement region depends on the
// likely subtags from the language and script subtags.

const testData = {
  // For example, the breakup of the Soviet Union ("SU") means that the region of
  // the Soviet Union ("SU") is replaced by Russia ("RU"), Armenia ("AM"), or
  // many others -- depending on the specified (or merely likely) language and
  // script subtags:
  //
  // <territoryAlias type="SU" replacement="RU AM AZ BY EE GE KZ KG LV LT MD TJ TM UA UZ" reason="deprecated"/>
  // <territoryAlias type="810" replacement="RU AM AZ BY EE GE KZ KG LV LT MD TJ TM UA UZ" reason="overlong"/>
  "ru-SU": "ru-RU",
  "ru-810": "ru-RU",
  "en-SU": "en-RU",
  "en-810": "en-RU",
  "und-SU": "und-RU",
  "und-810": "und-RU",
  "und-Latn-SU": "und-Latn-RU",
  "und-Latn-810": "und-Latn-RU",

  // Armenia can be the preferred region when the language is "hy" (Armenian) or
  // the script is "Armn" (Armenian).
  //
  // <likelySubtag from="hy" to="hy_Armn_AM"/>
  // <likelySubtag from="und_Armn" to="hy_Armn_AM"/>
  "hy-SU": "hy-AM",
  "hy-810": "hy-AM",
  "und-Armn-SU": "und-Armn-AM",
  "und-Armn-810": "und-Armn-AM",

  // <territoryAlias type="CS" replacement="RS ME" reason="deprecated"/>
  //
  // The following likely-subtags entries contain "RS" and "ME":
  //
  // <likelySubtag from="sr" to="sr_Cyrl_RS"/>
  // <likelySubtag from="sr_ME" to="sr_Latn_ME"/>
  // <likelySubtag from="und_RS" to="sr_Cyrl_RS"/>
  // <likelySubtag from="und_ME" to="sr_Latn_ME"/>
  //
  // In this case there is no language/script combination (without a region
  // subtag) where "ME" is ever chosen, so the replacement is always "RS".
  "sr-CS": "sr-RS",
  "sr-Latn-CS": "sr-Latn-RS",
  "sr-Cyrl-CS": "sr-Cyrl-RS",

  // The existing region in the source locale identifier is ignored when selecting
  // the likely replacement region. For example take "az-NT", which is Azerbaijani
  // spoken in the Neutral Zone. The replacement region for "NT" is either
  // "SA" (Saudi-Arabia) or "IQ" (Iraq), and there is also a likely subtags entry
  // for "az-IQ". But when only looking at the language subtag in "az-NT", "az" is
  // always resolved to "az-Latn-AZ", and because "AZ" is not in the list ["SA",
  // "IQ"], the final replacement region is the default for "NT", namely "SA".
  // That means "az-NT" will be canonicalised to "az-SA" and not "az-IQ", even
  // though the latter may be a more sensible candidate based on the actual usage
  // of the target locales.
  //
  // <territoryAlias type="NT" replacement="SA IQ" reason="deprecated"/>
  // <likelySubtag from="az_IQ" to="az_Arab_IQ"/>
  // <likelySubtag from="az" to="az_Latn_AZ"/>
  "az-NT": "az-SA",
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
