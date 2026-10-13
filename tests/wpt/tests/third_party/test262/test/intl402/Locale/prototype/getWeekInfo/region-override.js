// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getWeekInfo
description: >
  When the locale has a region override ("rg") keyword, its region overrides
  the region subtag when deriving the week info.
info: |
  WeekInfoOfLocale ( loc )
  ...
  2. Let _preference_ be RegionPreference(_loc_.[[Locale]]).
  3. Let _region_ be _preference_.[[Region]].
  4. Let _regionOverride_ be _preference_.[[RegionOverride]].
  5. If _regionOverride_ is not *undefined* and week data for _regionOverride_
     are available, then
    a. Let _lookupRegion_ be _regionOverride_.
  6. Else,
    a. Let _lookupRegion_ be _region_.
  ...

  RegionPreference ( locale )
  ...
  3. Let _regionOverride_ be CanonicalUnicodeSubdivision(_locale_, *"rg"*).
  4. Return { [[Region]]: _region_, [[RegionOverride]]: _regionOverride_ }.
features: [Intl.Locale, Intl.Locale-info]
---*/

// In order not to rely on a particular implementation's locale data, this test
// searches for a suitable locale with a region subtag together with a region
// override ("rg" extension) whose region would influence the supported week
// info.
//
// Each candidate below is a locale with a region override extension, together
// with a locale that names the override's region explicitly (e.g.
// "en-US-u-rg-inzzzz" overrides the region with "IN", so "en-IN"). Because the
// region override is set, WeekInfoOfLocale must use it as the lookup region
// instead of the region subtag. Its week info must therefore be identical to
// those of the paired override-region locale. An incorrect implementation might
// instead ignore the "rg" keyword and use the region subtag.
function weekInfoEqual(a, b) {
  return a.firstDay === b.firstDay && compareArray(a.weekend, b.weekend);
}

function findSuitableTestData() {
  const candidates = [
    ["en-US-u-rg-dezzzz", "en-DE"],
    ["en-US-u-rg-inzzzz", "en-IN"],
    ["en-US-u-rg-irzzzz", "en-IR"],
    ["en-US-u-rg-afzzzz", "en-AF"],
  ];

  for (const [overrideTag, regionTag] of candidates) {
    const weekInfoWithOverrideRegion = new Intl.Locale(regionTag).getWeekInfo();

    const baseName = overrideTag.replace(/-u-.*/, "");
    const baseLocale = new Intl.Locale(baseName);
    if (!weekInfoEqual(baseLocale.getWeekInfo(), weekInfoWithOverrideRegion)) {
      return [overrideTag, weekInfoWithOverrideRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [overrideTag, expectedWeekInfo] = findSuitableTestData();

const locale = new Intl.Locale(overrideTag);
const weekInfo = locale.getWeekInfo();
assert.sameValue(
  weekInfo.firstDay,
  expectedWeekInfo.firstDay,
  `getWeekInfo() for "${overrideTag}" should return firstDay that matches the override region`
);
assert.compareArray(
  weekInfo.weekend,
  expectedWeekInfo.weekend,
  `getWeekInfo() for "${overrideTag}" should return weekend that matches the override region`
);
