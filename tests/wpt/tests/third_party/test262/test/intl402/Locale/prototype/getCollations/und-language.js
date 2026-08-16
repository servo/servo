// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCollations
description: >
  Return value for tags not matching any available locale is hardcoded in spec
info: |
  CollationsOfLocale ( loc )

  2. Let _match_ be LookupMatchingLocaleByPrefix(
     %Intl.Collator%.[[AvailableLocales]], « _loc_.[[Locale]] »).
  3. If _match_ is not *undefined*, then
    ...
  4. Else,
    a. Let _list_ be « *"emoji"*, *"eor"* ».
features: [Intl.Locale,Intl.Locale-info]
---*/

var undLocales = [
  "und",
  "und-US",
  "und-Latn",
  "und-Latn-US",
  "und-u-ca-gregory",
  "und-US-u-nu-latn",
];

for (var i = 0; i < undLocales.length; i++) {
  var tag = undLocales[i];
  var locale = new Intl.Locale(tag);
  assert.sameValue(
    locale.language,
    "und",
    tag + " must have language subtag 'und'"
  );
  assert.compareArray(
    locale.getCollations(),
    ["emoji", "eor"],
    "getCollations() for " + tag + " must return the root collations"
  );
}

// Unreserved private-use language subtags (qfz..qtz) are guaranteed to have no
// semantics in any CLDR release, so we assume they will fall back to the
// hardcoded root collations.
// https://www.unicode.org/reports/tr35/tr35.html#Private_Use_Codes
//
// Note regarding normative change https://github.com/tc39/ecma402/pull/1072:
// this test might pass or fail if the normative change is not implemented,
// depending on what the environment's default locale is. It should always pass
// regardless of the environment's default locale if the normative change is
// implemented correctly. Try running it with LC_ALL=de in the environment, for
// example.

var privateUseTags = ["qfz", "qga-DE", "qgb-ES", "qgc-KR", "qtz-CN"];

for (var i = 0; i < privateUseTags.length; i++) {
  var tag = privateUseTags[i];
  assert.compareArray(
    new Intl.Locale(tag).getCollations(),
    ["emoji", "eor"],
    "getCollations() for unavailable locale " + tag + " must return the root collations"
  );
}
