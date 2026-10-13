// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getHourCycles
description: >
  The region used to look up the hour cycle preference is chosen from the
  available signals in the correct priority order.
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
  1. Let _region_ be GetLocaleRegion(_locale_).
  2. If _region_ is *undefined*, then
    a. Set _region_ to CanonicalUnicodeSubdivision(_locale_, *"sd"*).
    b. If _region_ is *undefined*, then
      i. Let _maximal_ be the result of the Add Likely Subtags algorithm applied
         to _locale_. If an error is signaled, set _maximal_ to _locale_.
      ii. Set _maximal_ to CanonicalizeUnicodeLocaleId(_maximal_).
      iii. Set _region_ to GetLocaleRegion(_maximal_).
      iv. If _region_ is *undefined*, then
        1. Set _region_ to *"001"*.
  3. Let _regionOverride_ be CanonicalUnicodeSubdivision(_locale_, *"rg"*).
  4. Return { [[Region]]: _region_, [[RegionOverride]]: _regionOverride_ }.
features: [Intl.Locale, Intl.Locale-info]
---*/

// Together, HourCyclesOfLocale and RegionPreference select the region used to
// look up the hour cycle preference from the available signals in this priority
// order (highest first):
//
//   1. the "rg" region override keyword (when time data for it exists)
//   2. the region subtag
//   3. the "sd" subdivision keyword
//   4. the region computed by the Add Likely Subtags algorithm
//   5. the "001" (world) region
//
// Each entry below is a locale that carries the signal for its priority level
// on top of all lower-priority signals, paired with a locale that names the
// region that signal must resolve to. The paired locale uses the same base
// language so that any language-dependent locale data is held constant. For
// example, "en-US-u-sd-gbeng-rg-gbzzzz" carries an "rg" override of region "GB",
// a region subtag "US", an "sd" subdivision in region "GB", and a base language
// "en" whose likely region is "US"; the "rg" override has the highest priority,
// so the locale must behave like "en-GB".
//
// The last entry has no lower-priority signals: the language "eo" (Esperanto)
// has no likely region, so its region falls back to "001".
const levels = [
  { description: 'the "rg" region override', locale: "en-US-u-sd-gbeng-rg-gbzzzz", region: "en-GB" },
  { description: "the region subtag", locale: "en-US-u-sd-gbeng", region: "en-US" },
  { description: 'the "sd" subdivision', locale: "en-u-sd-gbeng", region: "en-GB" },
  { description: "the Add Likely Subtags region", locale: "en", region: "en-US" },
  { description: 'the "001" region', locale: "eo", region: "eo-001" },
];

// For each priority level to be observable, the region it selects must have
// different hour cycles than the region selected by the next lower level in this
// implementation; otherwise the test could not tell them apart.
for (let i = 0; i < levels.length - 1; i++) {
  const higher = new Intl.Locale(levels[i].region).getHourCycles();
  const lower = new Intl.Locale(levels[i + 1].region).getHourCycles();
  assert(
    !compareArray(higher, lower),
    `Inconclusive: ${levels[i].description} and ${levels[i + 1].description} ` +
    "select regions with identical hour cycles in this implementation. Consider updating the test data"
  );
}

for (const { description, locale, region } of levels) {
  assert.compareArray(
    new Intl.Locale(locale).getHourCycles(),
    new Intl.Locale(region).getHourCycles(),
    `getHourCycles() for "${locale}" should use ${description}, like "${region}"`
  );
}
