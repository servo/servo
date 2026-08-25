// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies getters with grandfathered tags.
info: |
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
features: [Intl.Locale]
---*/

// Regular grandfathered language tag.
var loc = new Intl.Locale("cel-gaulish");
assert.sameValue(loc.baseName, "xtg");
assert.sameValue(loc.language, "xtg");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, undefined);

loc = new Intl.Locale("cel", { variants: "gaulish" });
assert.sameValue(loc.baseName, "xtg");
assert.sameValue(loc.language, "xtg");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, undefined);

// Regular grandfathered language tag.
assert.throws(RangeError, () => new Intl.Locale("zh-min"));

assert.throws(RangeError, () => new Intl.Locale("i-default"));

