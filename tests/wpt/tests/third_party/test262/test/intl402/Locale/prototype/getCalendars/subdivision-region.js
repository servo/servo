// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCalendars
description: >
  When the locale has no region but has a subdivision ("sd") keyword, the
  subdivision's region is used to derive the calendar preference.
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
// subdivision ("sd" extension) whose region would influence the supported
// calendars.
//
// Each candidate below is a locale with a subdivision extension but no region
// subtag, together with a locale that names that subdivision's region
// explicitly (e.g. "en-u-sd-th10" has subdivision "th10", whose region is "TH",
// so "en-TH"). Because such a locale has no region subtag, RegionPreference
// must derive its region from the subdivision. Its calendars must therefore be
// identical to those of the paired region locale. An incorrect implementation
// might instead ignore the "sd" extension and apply Add Likely Subtags to the
// bare language, giving "en-Latn-US".
function findSuitableTestData() {
  const candidates = [
    ["en-u-sd-th10", "en-TH"],
    ["en-u-sd-jp13", "en-JP"],
    ["en-u-sd-inka", "en-IN"],
    ["en-u-sd-irthr", "en-IR"],
  ];

  for (const [subdivisionTag, regionTag] of candidates) {
    const calendarsWithSubdivisionRegion = new Intl.Locale(regionTag).getCalendars();

    const bareLanguage = new Intl.Locale(subdivisionTag).baseName;
    const fallbackLocale = new Intl.Locale(bareLanguage).maximize();
    if (!compareArray(fallbackLocale.getCalendars(), calendarsWithSubdivisionRegion)) {
      return [subdivisionTag, calendarsWithSubdivisionRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [subdivisionTag, expectedCalendars] = findSuitableTestData();

const locale = new Intl.Locale(subdivisionTag);
assert.compareArray(
  locale.getCalendars(),
  expectedCalendars,
  `getCalendars() for "${subdivisionTag}" should equal getCalendars() for the subdivision's region`
);
