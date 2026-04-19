// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies getters with normal tags.
info: |
    Intl.Locale.prototype.toString ()
    3. Return loc.[[Locale]].

    get Intl.Locale.prototype.baseName
    3. Return GetLocaleBaseName(_loc_.[[Locale]]).

    GetLocaleBaseName
    2. Return the longest prefix of _locale_ matched by the
       <code>unicode_language_id</code> Unicode locale nonterminal.

    get Intl.Locale.prototype.language
    3. Return GetLocaleLanguage(_loc_.[[Locale]]).

    GetLocaleLanguage
    1. Let _baseName_ be GetLocaleBaseName(_locale_).
    2. Assert: The first subtag of _baseName_ can be matched by the
       <code>unicode_language_subtag</code> Unicode locale nonterminal.
    3. Return the first subtag of _baseName_.

    get Intl.Locale.prototype.script
    3. Return GetLocaleScript(_loc_.[[Locale]]).

    GetLocaleScript
    1. Let _baseName_ be GetLocaleBaseName(_locale_).
    2. Assert: _baseName_ contains at most one subtag that can be matched by the
       <code>unicode_script_subtag</code> Unicode locale nonterminal.
    3. If _baseName_ contains a subtag matched by the
       <code>unicode_script_subtag</code> Unicode locale nonterminal, return
       that subtag.
    4. Return *undefined*.

    get Intl.Locale.prototype.region
    3. Return GetLocaleRegion(_loc_.[[Locale]]).

    GetLocaleRegion
    1. Let _baseName_ be GetLocaleBaseName(_locale_).
    2. NOTE: A <code>unicode_region_subtag</code> subtag is only valid
       immediately after an initial <code>unicode_language_subtag</code> subtag,
       optionally with a single <code>unicode_script_subtag</code> subtag
       between them. In that position, <code>unicode_region_subtag</code> cannot
       be confused with any other valid subtag because all their productions are
       disjoint.
    3. Assert: The first subtag of _baseName_ can be matched by the
       <code>unicode_language_subtag</code> Unicode locale nonterminal.
    4. Let _baseNameTail_ be the suffix of _baseName_ following the first
       subtag.
    5. Assert: _baseNameTail_ contains at most one subtag that can be matched by
       the <code>unicode_region_subtag</code> Unicode locale nonterminal.
    6. If _baseNameTail_ contains a subtag matched by the
       <code>unicode_region_subtag</code> Unicode locale nonterminal, return
       that subtag.
    7. Return *undefined*.

    get Intl.Locale.prototype.variants
    3. Return GetLocaleVariants(_loc_.[[Locale]]).

    GetLocaleVariants
    1. Let _baseName_ be GetLocaleBaseName(_locale_).
    2. NOTE: Each subtag in _baseName_ that is preceded by *"-"* is either a
       <code>unicode_script_subtag</code>, <code>unicode_region_subtag</code>,
       or <code>unicode_variant_subtag</code>, but any substring matched by
       <code>unicode_variant_subtag</code> is strictly longer than any prefix
       thereof which could also be matched by one of the other productions.
    3. Let _variants_ be the longest suffix of _baseName_ that starts with a
       *"-"* followed by a <emu-not-ref>substring</emu-not-ref> that is matched
       by the <code>unicode_variant_subtag</code> Unicode locale nonterminal. If
       there is no such suffix, return *undefined*.
    4. Return the substring of _variants_ from 1.

    get Intl.Locale.prototype.calendar
    3. Return loc.[[Calendar]].

    get Intl.Locale.prototype.collation
    3. Return loc.[[Collation]].

    get Intl.Locale.prototype.hourCycle
    3. Return loc.[[HourCycle]].

    get Intl.Locale.prototype.caseFirst
    This property only exists if %Locale%.[[RelevantExtensionKeys]] contains "kf".
    3. Return loc.[[CaseFirst]].

    get Intl.Locale.prototype.numeric
    This property only exists if %Locale%.[[RelevantExtensionKeys]] contains "kn".
    3. Return loc.[[Numeric]].

    get Intl.Locale.prototype.numberingSystem
    3. Return loc.[[NumberingSystem]].

features: [Intl.Locale]
---*/

// Test all getters return the expected results.
var langtag = "de-latn-de-fonipa-1996-u-ca-gregory-co-phonebk-hc-h23-kf-true-kn-false-nu-latn";
var loc = new Intl.Locale(langtag);

assert.sameValue(loc.toString(), "de-Latn-DE-1996-fonipa-u-ca-gregory-co-phonebk-hc-h23-kf-kn-false-nu-latn");
assert.sameValue(loc.baseName, "de-Latn-DE-1996-fonipa");
assert.sameValue(loc.language, "de");
assert.sameValue(loc.script, "Latn");
assert.sameValue(loc.region, "DE");
assert.sameValue(loc.variants, "1996-fonipa");
assert.sameValue(loc.calendar, "gregory");
assert.sameValue(loc.collation, "phonebk");
assert.sameValue(loc.hourCycle, "h23");
if ("caseFirst" in loc) {
    assert.sameValue(loc.caseFirst, "");
}
if ("numeric" in loc) {
    assert.sameValue(loc.numeric, false);
}
assert.sameValue(loc.numberingSystem, "latn");

// Replace all components through option values and validate the getters still
// return the expected results.
var loc = new Intl.Locale(langtag, {
    language: "ja",
    script: "jpan",
    region: "jp",
    variants: "Hepburn",
    calendar: "japanese",
    collation: "search",
    hourCycle: "h24",
    caseFirst: "false",
    numeric: "true",
    numberingSystem: "jpanfin",
});

assert.sameValue(loc.toString(), "ja-Jpan-JP-hepburn-u-ca-japanese-co-search-hc-h24-kf-false-kn-nu-jpanfin");
assert.sameValue(loc.baseName, "ja-Jpan-JP-hepburn");
assert.sameValue(loc.language, "ja");
assert.sameValue(loc.script, "Jpan");
assert.sameValue(loc.region, "JP");
assert.sameValue(loc.variants, "hepburn");
assert.sameValue(loc.calendar, "japanese");
assert.sameValue(loc.collation, "search");
assert.sameValue(loc.hourCycle, "h24");
if ("caseFirst" in loc) {
    assert.sameValue(loc.caseFirst, "false");
}
if ("numeric" in loc) {
    assert.sameValue(loc.numeric, true);
}
assert.sameValue(loc.numberingSystem, "jpanfin");

// Replace only some components through option values and validate the getters
// return the expected results.
var loc = new Intl.Locale(langtag, {
    language: "fr",
    region: "ca",
    collation: "standard",
    hourCycle: "h11",
});

assert.sameValue(loc.toString(), "fr-Latn-CA-1996-fonipa-u-ca-gregory-co-standard-hc-h11-kf-kn-false-nu-latn");
assert.sameValue(loc.baseName, "fr-Latn-CA-1996-fonipa");
assert.sameValue(loc.language, "fr");
assert.sameValue(loc.script, "Latn");
assert.sameValue(loc.region, "CA");
assert.sameValue(loc.variants, "1996-fonipa");
assert.sameValue(loc.calendar, "gregory");
assert.sameValue(loc.collation, "standard");
assert.sameValue(loc.hourCycle, "h11");
if ("caseFirst" in loc) {
    assert.sameValue(loc.caseFirst, "");
}
if ("numeric" in loc) {
    assert.sameValue(loc.numeric, false);
}
assert.sameValue(loc.numberingSystem, "latn");

// Check that "und" language subtag returns "und" for `language` getter
var loc = new Intl.Locale("und");

assert.sameValue(loc.toString(), "und");
assert.sameValue(loc.baseName, "und");
assert.sameValue(loc.language, "und");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, undefined);

var loc = new Intl.Locale("und-US-u-co-emoji");

assert.sameValue(loc.toString(), "und-US-u-co-emoji");
assert.sameValue(loc.baseName, "und-US");
assert.sameValue(loc.language, "und");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, "US");
assert.sameValue(loc.variants, undefined);
if ("collation" in loc) {
    assert.sameValue(loc.collation, "emoji");
}
