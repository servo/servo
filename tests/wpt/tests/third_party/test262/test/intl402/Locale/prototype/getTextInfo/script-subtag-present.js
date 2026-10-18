// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Determine text direction from script subtag, ignore the language subtag.
info: |
  TextDirectionOfLocale ( loc )
  ...
  2. Let script be GetLocaleScript(locale).
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

const languages = [
  // Undetermined language.
  "und",

  // English as a sample language whose default script is left-to-right.
  "en",

  // Arabic as a sample language whose default script is right-to-left.
  "ar",

  // Klingon as a sample constructed language.
  "tlh",

  // "qfz" as a CLDR private use, excluded language subtag.
  "qfz",
];

const scripts = {
  // Latin script is written left-to-right.
  Latn: "ltr",

  // Arabic script is written right-to-left.
  Arab: "rtl",
};

for (let [script, direction] of Object.entries(scripts)) {
  for (let language of languages) {
    let locale = `${language}-${script}`;

    assert.sameValue(
      new Intl.Locale(locale).getTextInfo().direction,
      direction,
      `with locale "${locale}"`
    );
  }
}
