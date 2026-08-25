// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getHourCycles
description: >
  When the locale has no region but has a subdivision ("sd") keyword, the
  subdivision's region is used to derive the hour cycle preference.
info: |
  RegionPreference ( locale )
  1. Let _region_ be GetLocaleRegion(_locale_).
  2. If _region_ is *undefined*, then
    a. Set _region_ to CanonicalUnicodeSubdivision(_locale_, *"sd"*).
    b. If _region_ is *undefined*, then
      i. Let _maximal_ be the result of the Add Likely Subtags algorithm applied
         to _locale_. If an error is signaled, set _maximal_ to _locale_.
      ...
  ...
features: [Intl.Locale, Intl.Locale-info]
---*/

// In order not to rely on a particular implementation's locale data, this test
// searches for a suitable locale without a region subtag, but with a
// subdivision ("sd" extension) whose region would influence the supported hour
// cycles.
//
// Each candidate below is a locale with a subdivision extension but no region
// subtag, together with a locale that names that subdivision's region
// explicitly (e.g. "en-u-sd-gbeng" has subdivision "gbeng", whose region is
// "GB", so "en-GB"). Because such a locale has no region subtag,
// RegionPreference must derive its region from the subdivision. Its hour cycles
// must therefore be identical to those of the paired region locale. An
// incorrect implementation might instead ignore the "sd" extension and apply
// Add Likely Subtags to the bare language, giving "en-Latn-US".
function findSuitableTestData() {
  const candidates = [
    ["en-u-sd-gbeng", "en-GB"],
    ["en-u-sd-fridf", "en-FR"],
    ["en-u-sd-debe", "en-DE"],
  ];

  for (const [subdivisionTag, regionTag] of candidates) {
    const hourCyclesWithSubdivisionRegion = new Intl.Locale(regionTag).getHourCycles();

    const bareLanguage = new Intl.Locale(subdivisionTag).baseName;
    const fallbackLocale = new Intl.Locale(bareLanguage).maximize();
    if (!compareArray(fallbackLocale.getHourCycles(), hourCyclesWithSubdivisionRegion)) {
      return [subdivisionTag, hourCyclesWithSubdivisionRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [subdivisionTag, expectedHourCycles] = findSuitableTestData();

const locale = new Intl.Locale(subdivisionTag);
assert.compareArray(
  locale.getHourCycles(),
  expectedHourCycles,
  `getHourCycles() for "${subdivisionTag}" should equal getHourCycles() for the subdivision's region`
);
