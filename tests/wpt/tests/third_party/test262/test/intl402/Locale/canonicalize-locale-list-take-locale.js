// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies CanonicalizeLocaleList will take Intl.Locale as locales.
info: |
    CanonicalizeLocaleList ( locales )
    3. If Type(locales) is String or locales has an [[InitializedLocale]] internal slot, then
       a. Let O be CreateArrayFromList(« locales »).

       c. iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
            1. Let tag be kValue.[[Locale]].
          iv. Else,
            1. Let tag be ? ToString(kValue).
features: [Intl.Locale]
---*/

const tag = "ar";
const tag2 = "fa";
const tag3 = "zh";
const loc = new Intl.Locale(tag);

// Monkey-patching Intl.Locale
class PatchedLocale extends Intl.Locale {
  constructor(tag, options) {
    super(tag, options);
  }
  toString() {
    // this should NOT get called.
    assert(false, "toString should not be called")
  }
}
const ploc = new PatchedLocale(tag2);

// Test Intl.Locale as the only argument
let res = Intl.getCanonicalLocales(loc);
assert.sameValue(res.length, 1);
assert.sameValue(res[0], tag);

// Test Monkey-patched Intl.Locale as the only argument
res = Intl.getCanonicalLocales(ploc);
assert.sameValue(res.length, 1);
assert.sameValue(res[0], tag2);

// Test Intl.Locale and the Monkey-patched one are in
// array.
res = Intl.getCanonicalLocales([loc, tag3, ploc]);
assert.sameValue(res.length, 3);
assert.sameValue(res[0], tag);
assert.sameValue(res[1], tag3);
assert.sameValue(res[2], tag2);
