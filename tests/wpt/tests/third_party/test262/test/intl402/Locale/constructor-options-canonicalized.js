// Copyright 2020 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-resolvelocale
description: >
    Values provided as properties of the options-argument to the Locale
    constructor are converted to canonical form.
info: |
    ResolveLocale ( availableLocales, requestedLocales, options, relevantExtensionKeys, localeData )

    ...
    9.i.iii.1. Let optionsValue be the string optionsValue after performing the algorithm steps to transform Unicode extension values to canonical syntax per Unicode Technical Standard #35 LDML § 3.2.1 Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    9.i.iii.2. Let optionsValue be the string optionsValue after performing the algorithm steps to replace Unicode extension values with their canonical form per Unicode Technical Standard #35 LDML § 3.2.1 Canonical Unicode Locale Identifiers, treating key as ukey and optionsValue as uvalue productions.
    ...

features: [Intl.Locale]
---*/

const keyValueTests = [
  {
    key: "ca",
    option: "calendar",
    tests: [
      ["islamicc", "islamic-civil"],
      ["ethiopic-amete-alem", "ethioaa"],
    ],
  },
];

for (const { key, option, tests } of keyValueTests) {
  for (const [noncanonical, canonical] of tests) {
    let canonicalInLocale =
      new Intl.Locale(`en-u-${key}-${canonical}`);

    assert.sameValue(
      canonicalInLocale[option],
      canonical,
      `new Intl.Locale("en-u-${key}-${canonical}").${option} returns ${canonical}`
    );

    let canonicalInOption =
      new Intl.Locale(`en`, { [option]: canonical });

    assert.sameValue(
      canonicalInOption[option],
      canonical,
      `new Intl.Locale("en", { ${option}: "${canonical}" }).${option} returns ${canonical}`
    );

    let noncanonicalInLocale =
      new Intl.Locale(`en-u-${key}-${noncanonical}`);

    assert.sameValue(
      noncanonicalInLocale[option],
      canonical,
      `new Intl.Locale("en-u-${key}-${noncanonical}").${option} returns ${canonical}`
    );

    let noncanonicalInOption =
      new Intl.Locale(`en`, { [option]: noncanonical });

    assert.sameValue(
      noncanonicalInOption[option],
      canonical,
      `new Intl.Locale("en", { ${option}: "${noncanonical}" }).${option} returns ${canonical}`
    );
  }
}
