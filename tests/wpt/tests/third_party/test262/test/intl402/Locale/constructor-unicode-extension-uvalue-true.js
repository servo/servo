// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
  Remove any uvalue (aka type) equal to "true".
info: |
  Intl.Locale ( tag [ , options ] )
    ...
    13. Set tag to CanonicalizeUnicodeLocaleId(tag).
    ...
    35. Let r be MakeLocaleRecord(tag, opt, localeExtensionKeys).
    ...
features: [Intl.Locale]
---*/

// Generate all possible `ukey` values.
// https://unicode.org/reports/tr35/#ukey
function* ukeys() {
  const lowerAlpha = 'abcdefghijklmnopqrstuvwxyz';
  const lowerAlphanum = lowerAlpha + '0123456789';

  for (let i = 0; i < lowerAlphanum.length; ++i) {
    for (let j = 0; j < lowerAlpha.length; ++j) {
      yield lowerAlphanum[i] + lowerAlpha[j];
    }
  }
}

for (let ukey of ukeys()) {
  const localeId = `en-u-${ukey}-true`;
  const normalizedLocaleId = `en-u-${ukey}`;
  assert.sameValue(
    new Intl.Locale(localeId).toString(),
    normalizedLocaleId,
    `new Intl.Locale("${localeId}").toString() must be "${normalizedLocaleId}"`
  );
}
