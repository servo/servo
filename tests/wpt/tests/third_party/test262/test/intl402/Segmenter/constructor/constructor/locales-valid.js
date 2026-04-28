// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks various cases for the locales argument to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    3. Let _requestedLocales_ be ? CanonicalizeLocaleList(_locales_).
features: [Intl.Segmenter, Array.prototype.includes]
---*/

const defaultLocale = new Intl.Segmenter().resolvedOptions().locale;

const tests = [
  [undefined, [defaultLocale], "undefined"],
  ["EN", ["en"], "Single value"],
  [[], [defaultLocale], "Empty array"],
  [["sr"], ["sr"], "Single-element array"],
  [["fr", "ar"], ["fr", "ar"], "Two-element array"],
  [["xyz", "ar"], ["ar"], "Two-element array with unknown code"],
  [["en", "EN"], ["en"], "Duplicate value (canonical first)"],
  [["EN", "en"], ["en"], "Duplicate value (canonical last)"],
  [{ 0: "DE", length: 0 }, [defaultLocale], "Object with zero length"],
  [{ 0: "DE", length: 1 }, ["de"], "Object with length"],
  [{ 0: "ja", 1: "fr" }, [defaultLocale], "Object without length, indexed from 0"],
  [{ 1: "ja", 2: "fr" }, [defaultLocale], "Object without length, indexed from 1"],
];

const errorTests = [
  [["en-GB-oed"], "Grandfathered"],
  [["x-private"], "Private", ["lookup"]],
];

for (const [locales, expected, name, matchers = ["best fit", "lookup"]] of tests) {
  for (const localeMatcher of matchers) {
    const segmenter = new Intl.Segmenter(locales, { localeMatcher });
    const actual = segmenter.resolvedOptions().locale;
    assert(expected.includes(actual), `${name}: expected one of ${expected}, found ${actual}`);
  }
}

for (const [locales, name, matchers = ["best fit", "lookup"]] of errorTests) {
  for (const localeMatcher of matchers) {
    assert.throws(RangeError, function() {
      new Intl.Segmenter(locales, { localeMatcher });
    }, name);
  }
}
