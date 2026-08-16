// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getHourCycles
description: >
  The language subtag is taken into account when looking up the hour cycle
  preference
info: |
  HourCyclesOfLocale ( loc )
  ...
  5. Let _language_ be GetLocaleLanguage(_loc_.[[Locale]]).
  6. For each String _region_ of _preferredRegions_, do
    a. Let _locale_ be the string-concatenation of _language_, *"-"*, and
       _region_.
    b. If _hourCycles_ is empty and time data for locale _locale_ are available,
       then
      i. Set _hourCycles_ to a List of unique hour cycle identifiers, which must
         be lower case Strings indicating either the 12-hour format (*"h11"*,
         *"h12"*) or the 24-hour format (*"h23"*, *"h24"*), sorted in descending
         preference of those in common use for date and time formatting in
         locale _locale_.
    c. If _hourCycles_ is empty and time data for region _region_ are available,
       then
      i. Set _hourCycles_ to a List of unique hour cycle identifiers, which must
         be lower case Strings indicating either the 12-hour format (*"h11"*,
         *"h12"*) or the 24-hour format (*"h23"*, *"h24"*), sorted in descending
         preference of those in common use for date and time formatting in
         region _region_.
  ...
features: [Intl.Locale, Intl.Locale-info]
---*/

// Some of CLDR's hour cycle preference data is indexed by language and region
// in addition to the region-only entries. When such language-region data are
// available, getHourCycles() must use them in preference to the region-only
// data. For example, "fr-CA" is listed as preferring a 24-hour clock while
// "en-CA" is listed as preferring a 12-hour clock, so these locales must return
// different hour cycles even though they share the region "CA".
//
// To avoid relying on a particular implementation's locale data, the test tries
// several candidates. Each candidate below is a region paired with two
// languages where language-region time data exists for one. One of them should
// return different hour-cycle data than the control locale consisting of
// language "und". An implementation that ignored the language subtag would look
// up only the region and make all three locales agree.
const candidates = [
  { region: "CA", languages: ["fr", "en"] },
  { region: "SY", languages: ["ku", "ar"] },
  { region: "001", languages: ["en", "de"] },
  { region: "001", languages: ["ar", "fr"] },
];

let found = false;
for (const { region, languages: [lang1, lang2] } of candidates) {
  const control = new Intl.Locale(`und-${region}`);
  const hourCycles = control.getHourCycles();
  const locale1 = new Intl.Locale(`${lang1}-${region}`);
  const locale2 = new Intl.Locale(`${lang2}-${region}`);
  const lang1Different = !compareArray(locale1.getHourCycles(), hourCycles);
  const lang2Different = !compareArray(locale2.getHourCycles(), hourCycles);
  if (lang1Different || lang2Different) {
    found = true;
    break;
  }
}

assert(found, "Either the feature is not implemented correctly, or no suitable test data found in this implementation. If the latter, consider updating the candidate list");
