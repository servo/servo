// Copyright (C) 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-makelocalerecord
description: >
  MakeLocaleRecord calls CanonicalizeUValue for option values
info: |
  MakeLocaleRecord ( tag, options, localeExtensionKeys )

  ...
  4. For each element key of localeExtensionKeys, do
    ...
    d. Let overrideValue be options.[[<key>]].
    e. If overrideValue is not undefined, then
      i. Set value to CanonicalizeUValue(key, overrideValue).
      ii.If entry is not empty, then
        1. Set entry.[[Value]] to value.
      iii. Else,
        1. Append the Record { [[Key]]: key, [[Value]]: value } to keywords.
    f. Set result.[[<key>]] to value.
  ...
features: [Intl.Locale]
---*/

const locales = [
  // Unicode extension "nu" not present in locale id.
  "de",

  // Unicode extension "nu" is present in locale id.
  "de-u-nu-thai",
];

const numberingSystems = [
  // Known and valid value for Unicode extension "nu".
  "latn",

  // Unknown and invalid value for Unicode extension "nu".
  "unknown",
];

for (let locale of locales) {
  for (let numberingSystem of numberingSystems) {
    let uppercase = numberingSystem.toUpperCase();
    let loc = new Intl.Locale(locale, {numberingSystem: uppercase});

    assert.sameValue(
      loc.numberingSystem,
      numberingSystem,
      `new Intl.Locale("${locale}", {numberingSystem: "${uppercase}"}).numberingSystem`
    );
  }
}
