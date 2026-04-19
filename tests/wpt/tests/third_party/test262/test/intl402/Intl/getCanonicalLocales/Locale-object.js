// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: Tests for Locale objects in the argument to getCanonicalLocales
info: |
    CanonicalizeLocaleList ( locales )
    7. c. iii. If Type(kValue) is Object and kValue has an [[InitializedLocale]] internal slot, then
                1. Let tag be kValue.[[Locale]].
includes: [compareArray.js]
features: [Intl.Locale]
---*/

assert.compareArray(Intl.getCanonicalLocales([
  "fr-CA",
  new Intl.Locale("en-gb-oxendict"),
  "de",
  new Intl.Locale("jp", { "calendar": "gregory" }),
  "zh",
  new Intl.Locale("fr-CA"),
]), [
  "fr-CA",
  "en-GB-oxendict",
  "de",
  "jp-u-ca-gregory",
  "zh",
]);
