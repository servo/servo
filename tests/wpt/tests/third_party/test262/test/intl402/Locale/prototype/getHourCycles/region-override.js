// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getHourCycles
description: >
  When the locale has a region override ("rg") keyword, its region overrides
  the region subtag when deriving the hour cycle preference.
info: |
  HourCyclesOfLocale ( loc )
  ...
  2. Let _preference_ be RegionPreference(_loc_.[[Locale]]).
  3. Let _region_ be _preference_.[[Region]].
  4. Let _regionOverride_ be _preference_.[[RegionOverride]].
  5. If _regionOverride_ is not *undefined* and time data for _regionOverride_
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
// override ("rg" extension) whose region would influence the supported hour
// cycles.
//
// Each candidate below is a locale with a region override extension, together
// with a locale that names the override's region explicitly (e.g.
// "en-US-u-rg-gbzzzz" overrides the region with "GB", so "en-GB"). Because the
// region override is set, HourCyclesOfLocale must use it as the lookup region
// instead of the region subtag. Its hour cycles must therefore be identical to
// those of the paired override-region locale. An incorrect implementation might
// instead ignore the "rg" keyword and use the region subtag.
function findSuitableTestData() {
  const candidates = [
    ["en-US-u-rg-gbzzzz", "en-GB"],
    ["en-US-u-rg-dezzzz", "en-DE"],
    ["en-US-u-rg-frzzzz", "en-FR"],
  ];

  for (const [overrideTag, regionTag] of candidates) {
    const hourCyclesWithOverrideRegion = new Intl.Locale(regionTag).getHourCycles();

    const withoutOverride = new Intl.Locale(new Intl.Locale(overrideTag).baseName);
    if (!compareArray(withoutOverride.getHourCycles(), hourCyclesWithOverrideRegion)) {
      return [overrideTag, hourCyclesWithOverrideRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [overrideTag, expectedHourCycles] = findSuitableTestData();

const locale = new Intl.Locale(overrideTag);
assert.compareArray(
  locale.getHourCycles(),
  expectedHourCycles,
  `getHourCycles() for "${overrideTag}" should equal getHourCycles() for the override region`
);
