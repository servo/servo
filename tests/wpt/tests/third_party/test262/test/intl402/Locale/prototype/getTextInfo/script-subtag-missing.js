// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Add likely subtags if the script subtag is missing.
info: |
  TextDirectionOfLocale ( loc )
  ...
  2. Let script be GetLocaleScript(locale).
  3. If script is undefined, then
    a. Let maximal be the result of the Add Likely Subtags algorithm applied to locale. If an error is signaled, return undefined.
    b. Set script to GetLocaleScript(maximal).
    c. If script is undefined, return undefined.
  4. If the default general ordering of characters within a line in script is right-to-left, return "rtl".
  5. If the default general ordering of characters within a line in script is left-to-right, return "ltr".
  6. Return undefined.

  NOTE 1:
  When the direction of default general ordering of characters within a line in
  the script cannot be determined, or the direction is neither right-to-left nor
  left-to-right, then undefined will be returned.
features: [Intl.Locale, Intl.Locale-info]
---*/

const locales = [
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

  // Region subtag can alter likely subtags: "pa" maximizes to "pa-Guru-IN",
  // whereas "pa-PK" maximizes to "pa-Arab-PK". (Gurmukhi is written
  // left-to-right, Arabic is written right-to-left.)
  "pa",
  "pa-PK",
];

for (let locale of locales) {
  let loc = new Intl.Locale(locale);
  let maximized = loc.maximize();

  let direction = loc.getTextInfo().direction;
  assert.sameValue(
    direction,
    maximized.getTextInfo().direction,
    `same direction as maximized locale, with locale "${locale}"`
  );

  if (maximized.script === undefined) {
    assert.sameValue(
      direction,
      undefined,
      `maximized script is undefined for locale "${locale}"`
    );
  }
}
