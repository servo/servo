// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks  cases for the locales argument to the DurationFormat constructor.
info: |
    Intl.DurationFormat  ( [ locales [ , options ] ] )
    (...)
    3. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
features: [Intl.DurationFormat]
---*/

const defaultLocale = new Intl.DurationFormat().resolvedOptions().locale;

const matchers = ["lookup", "best fit"]

const tests = [
  [undefined, defaultLocale, "undefined"],
  ["EN", "en", "Single value"],
  [[], defaultLocale, "Empty array"],
  [["en", "EN"], "en", "Duplicate value (canonical first)"],
  [["EN", "en"], "en", "Duplicate value (canonical last)"],
  [{ 0: "DE", length: 0 }, defaultLocale, "Object with zero length"],
  [{ 0: "DE", length: 1 }, "de", "Object with length"],
];


for (const [locales, expected, name] of tests) {
  matchers.forEach((matcher)=>{
    const drf = new Intl.DurationFormat(locales, {localeMatcher: matcher});
    assert.sameValue(drf.resolvedOptions().locale, expected, name);
  });
};
