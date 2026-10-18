// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Returns undefined if the RTL field in scriptMetadata.txt is UNKNOWN.
info: |
  TextDirectionOfLocale ( loc )
  ...
  4. If the default general ordering of characters within a line in script is right-to-left, return "rtl".
  5. If the default general ordering of characters within a line in script is left-to-right, return "ltr".
  6. Return undefined.

  NOTE 1:
  When the direction of default general ordering of characters within a line in
  the script cannot be determined, or the direction is neither right-to-left nor
  left-to-right, then undefined will be returned.
features: [Intl.Locale, Intl.Locale-info]
---*/

// All scripts whose RTL field in scriptMetadata.txt is UNKNOWN.
const scripts = [
  "Zyyy",
  "Zinh",
  "Zzzz",
  "Brai",
];

const languages = [
  // Undetermined language.
  "und",

  // English as a sample language whose default script is left-to-right.
  "en",

  // Arabic as a sample language whose default script is right-to-left.
  "ar",

  // Klingon as sample constructed language.
  "tlh",

  // "qfz" as a CLDR private use, excluded language subtag.
  "qfz",
];

for (let script of scripts) {
  for (let language of languages) {
    let locale = `${language}-${script}`;

    assert.sameValue(
      new Intl.Locale(locale).getTextInfo().direction,
      undefined,
      `with locale "${locale}"`
    );
  }
}
