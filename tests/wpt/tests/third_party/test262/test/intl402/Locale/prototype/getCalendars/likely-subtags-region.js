// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCalendars
description: >
  When the locale has no region, the Add Likely Subtags algorithm is used to
  derive the region whose calendar preference is returned.
info: |
  RegionPreference ( locale )
  ...
  2. If _region_ is *undefined*, then
    a. Set _region_ to CanonicalUnicodeSubdivision(_locale_, *"sd"*).
    b. If _region_ is *undefined*, then
      i. Let _maximal_ be the result of the Add Likely Subtags algorithm applied
         to _locale_. If an error is signaled, set _maximal_ to _locale_.
      ii. Set _maximal_ to CanonicalizeUnicodeLocaleId(_maximal_).
      iii. Set _region_ to GetLocaleRegion(_maximal_).
      iv. If _region_ is *undefined*, then
        1. Set _region_ to *"001"*.
  ...
features: [Intl.Locale, Intl.Locale-info]
---*/

// In order not to rely on a particular implementation's locale data, this test
// searches for a suitable locale without a region subtag, but where the likely
// region would influence the supported calendars.
//
// Each candidate below is a language subtag together with the region that the
// Add Likely Subtags algorithm derives for it (e.g. "th" maximizes to
// "th-Thai-TH", so "th-TH"). A language tag without a region, subdivision ("sd"
// keyword), or region override ("rg" keyword) passed to RegionPreference must
// apply Add Likely Subtags and arrive at the paired region. Its calendars must
// therefore be identical to those of the candidate locale. An incorrect
// implementation might instead fall back to the region "001" in step 2.b.iv.
//
// We could assume even less about the locale data by not hardcoding these
// likely regions and instead checking all available locales with the result of
// maximize() on each one, but that assumes the implementation has a working
// maximize(), and if that's the case then it probably implements this step
// correctly as well.
function findSuitableTestData() {
  const candidates = ["th-TH", "fa-IR", "ja-JP", "sa-IN", "ps-AF"];

  for (const candidate of candidates) {
    const locale = new Intl.Locale(candidate);
    const calendarsWithLikelyRegion = locale.getCalendars();

    const bareLanguage = locale.language;
    const fallbackLocale = new Intl.Locale(`${bareLanguage}-001`);
    if (!compareArray(fallbackLocale.getCalendars(), calendarsWithLikelyRegion)) {
      return [bareLanguage, calendarsWithLikelyRegion];
    }
  }

  assert(false, "No suitable test data found in this implementation. Consider updating the candidate list");
}

const [language, expectedCalendars] = findSuitableTestData();

const locale = new Intl.Locale(language);
assert.compareArray(
  locale.getCalendars(),
  expectedCalendars,
  `getCalendars() for "${language}" should equal getCalendars() for locale with likely region`
);
