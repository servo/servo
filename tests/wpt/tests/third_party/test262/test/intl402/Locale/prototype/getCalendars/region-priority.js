// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCalendars
description: >
  The region used to look up the calendar preference is chosen from the
  available signals in the correct priority order.
info: |
  CalendarsOfLocale ( loc )
  ...
  2. Let _preference_ be RegionPreference(_loc_.[[Locale]]).
  3. Let _region_ be _preference_.[[Region]].
  4. Let _regionOverride_ be _preference_.[[RegionOverride]].
  5. If _regionOverride_ is not *undefined* and calendar preference data for
     _regionOverride_ are available, then
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

// Together, CalendarsOfLocale and RegionPreference select the region used to
// look up the calendar preference from the available signals in this priority
// order (highest first):
//
//   1. the "rg" region override keyword (when calendar data for it exists)
//   2. the region subtag
//   3. the "sd" subdivision keyword
//   4. the region computed by the Add Likely Subtags algorithm
//   5. the "001" (world) region
//
// Each entry below is a locale that carries the signal for its priority level
// on top of all lower-priority signals, paired with a locale that names the
// region that signal must resolve to. The paired locale uses the same base
// language so that any language-dependent locale data is held constant. For
// example, "fa-JP-u-sd-inka-rg-thzzzz" carries an "rg" override of region "TH",
// a region subtag "JP", an "sd" subdivision in region "IN", and a base language
// "fa" whose likely region is "IR"; the "rg" override has the highest priority,
// so the locale must behave like "fa-TH".
//
// The last entry has no lower-priority signals: the language "eo" (Esperanto)
// has no likely region, so its region falls back to "001".
const levels = [
  { description: 'the "rg" region override', locale: "fa-JP-u-sd-inka-rg-thzzzz", region: "fa-TH" },
  { description: "the region subtag", locale: "fa-JP-u-sd-inka", region: "fa-JP" },
  { description: 'the "sd" subdivision', locale: "fa-u-sd-inka", region: "fa-IN" },
  { description: "the Add Likely Subtags region", locale: "fa", region: "fa-IR" },
  { description: 'the "001" region', locale: "eo", region: "eo-001" },
];

// For each priority level to be observable, the region it selects must have
// different calendars than the region selected by the next lower level in this
// implementation; otherwise the test could not tell them apart.
for (let i = 0; i < levels.length - 1; i++) {
  const higher = new Intl.Locale(levels[i].region).getCalendars();
  const lower = new Intl.Locale(levels[i + 1].region).getCalendars();
  assert(
    !compareArray(higher, lower),
    `Inconclusive: ${levels[i].description} and ${levels[i + 1].description} ` +
    "select regions with identical calendars in this implementation. Consider updating the test data"
  );
}

for (const { description, locale, region } of levels) {
  assert.compareArray(
    new Intl.Locale(locale).getCalendars(),
    new Intl.Locale(region).getCalendars(),
    `getCalendars() for "${locale}" should use ${description}, like "${region}"`
  );
}
