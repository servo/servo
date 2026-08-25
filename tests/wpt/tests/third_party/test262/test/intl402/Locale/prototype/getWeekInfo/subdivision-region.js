// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getWeekInfo
description: >
  When the locale has no region but has a subdivision ("sd") keyword, the
  subdivision's region is used to derive the week info.
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
// subdivision ("sd" extension) whose region would influence the supported week
// info.
//
// Each candidate below is a locale with a subdivision extension but no region
// subtag, together with a locale that names that subdivision's region
// explicitly (e.g. "en-u-sd-inka" has subdivision "inka", whose region is "IN",
// so "en-IN"). Because such a locale has no region subtag, RegionPreference
// must derive its region from the subdivision. Its week info must therefore be
// identical to those of the paired region locale. An incorrect implementation
// might instead ignore the "sd" extension and apply Add Likely Subtags to the
// bare language, giving "en-Latn-US".
function weekInfoEqual(a, b) {
  return a.firstDay === b.firstDay && compareArray(a.weekend, b.weekend);
}

function findSuitableTestData() {
  const candidates = [
    ["en-u-sd-inka", "en-IN"],
    ["en-u-sd-irthr", "en-IR"],
    ["en-u-sd-afgh", "en-AF"],
  ];

  for (const [subdivisionTag, regionTag] of candidates) {
    const weekInfoWithSubdivisionRegion = new Intl.Locale(regionTag).getWeekInfo();

    const baseName = subdivisionTag.replace(/-u-.*/, "");
    const fallbackLocale = new Intl.Locale(baseName).maximize();
    if (!weekInfoEqual(fallbackLocale.getWeekInfo(), weekInfoWithSubdivisionRegion)) {
      return [subdivisionTag, weekInfoWithSubdivisionRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [subdivisionTag, expectedWeekInfo] = findSuitableTestData();

const locale = new Intl.Locale(subdivisionTag);
const weekInfo = locale.getWeekInfo();
assert.sameValue(
  weekInfo.firstDay,
  expectedWeekInfo.firstDay,
  `getWeekInfo() for "${subdivisionTag}" should return firstDay that matches the subdivision's region`
);
assert.compareArray(
  weekInfo.weekend,
  expectedWeekInfo.weekend,
  `getWeekInfo() for "${subdivisionTag}" should return weekend that matches the subdivision's region`
);
