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

function canonicalizeUValue(ukey, uvalue) {
  assert.sameValue(uvalue, "yes", "unexpected uvalue");

  switch (ukey) {
    case "kb":
    case "kc":
    case "kh":
    case "kk":
    case "kn":
      // Canonicalized to "true", which then gets removed.
      return "";
    default:
      return uvalue;
  }
}

for (let ukey of ukeys()) {
  let canonicalized = new Intl.Locale(`en-u-${ukey}-yes`).toString();

  if (canonicalizeUValue(ukey, "yes") === "yes") {
    assert.sameValue(
      canonicalized,
      `en-u-${ukey}-yes`,
      `new Intl.Locale("en-u-${ukey}-yes").toString() returns "en-u-${ukey}-yes"`
    );
  } else {
    assert.sameValue(
      canonicalized,
      `en-u-${ukey}`,
      `new Intl.Locale("en-u-${ukey}-yes").toString() returns "en-u-${ukey}"`
    );
  }
}
