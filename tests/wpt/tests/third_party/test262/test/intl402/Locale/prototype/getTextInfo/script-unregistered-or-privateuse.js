// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Returns undefined for unregistered and private-use script codes.
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

const scripts = [
  // "Aaaa" is not a registered script code.
  //
  // https://www.unicode.org/iso15924/iso15924-codes.html
  "Aaaa",

  // Use the private use script code "Qaaq", because it's guaranteed to never
  // get particular semantics in CLDR.
  //
  // https://unicode.org/reports/tr35/tr35.html#Private_Use_Codes
  "Qaaq",

  // Exceptionally reserved by ISO 15924, will not be assigned for any purpose.
  "True",
];

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
