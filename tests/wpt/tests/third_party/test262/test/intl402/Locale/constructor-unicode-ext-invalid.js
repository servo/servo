// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies treatment of specific structurally invalid tags.
info: |
    ApplyOptionsToTag( tag, options )
    2. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
features: [Intl.Locale]
---*/

const invalidLanguageTags = [
  // Unicode extension sequence is incomplete.
  "da-u",
  "da-u-",
  "da-u--",
  "da-u-t-latn",
  "da-u-x-priv",

  // Duplicate 'u' singleton.
  "da-u-ca-gregory-u-ca-buddhist"
];

for (const langtag of invalidLanguageTags) {
  assert.throws(RangeError, function() {
    new Intl.Locale(langtag)
  },
  `new Intl.Locale("${langtag}") throws RangeError`);
}
