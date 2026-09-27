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
function* ukeys() {
  const lowerA = 'a'.charCodeAt(0);
  const lowerZ = 'z'.charCodeAt(0);

  for (let first = lowerA; first <= lowerZ; ++first) {
    for (let second = lowerA; second <= lowerZ; ++second) {
      yield String.fromCharCode(first, second);
    }
  }
}

for (let ukey of ukeys()) {
  assert.sameValue(
    new Intl.Locale(`en-u-${ukey}-true`).toString(),
    `en-u-${ukey}`,
    `new Intl.Locale("en-u-${ukey}-true").toString() returns "en-u-${ukey}"`
  );
}
