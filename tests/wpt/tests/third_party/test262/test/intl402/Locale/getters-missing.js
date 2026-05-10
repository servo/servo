// Copyright 2018 André Bargull; Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies getters with missing tags.
info: |
    get Intl.Locale.prototype.baseName
    5. Return the substring of locale corresponding to the
       language ["-" script] ["-" region] *("-" variant)
       subsequence of the langtag grammar.

    get Intl.Locale.prototype.language
    4. Return the substring of locale corresponding to the language production.

    get Intl.Locale.prototype.script
    6. If locale does not contain the ["-" script] sequence, return undefined.
    7. Return the substring of locale corresponding to the script production.

    get Intl.Locale.prototype.region
    6. If locale does not contain the ["-" region] sequence, return undefined.
    7. Return the substring of locale corresponding to the region production.
features: [Intl.Locale]
---*/

// 'script' and 'region' subtags not present.
var loc = new Intl.Locale("sv");
assert.sameValue(loc.baseName, "sv");
assert.sameValue(loc.language, "sv");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, undefined);

// 'region' subtag not present.
var loc = new Intl.Locale("sv-Latn");
assert.sameValue(loc.baseName, "sv-Latn");
assert.sameValue(loc.language, "sv");
assert.sameValue(loc.script, "Latn");
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, undefined);

// 'script' subtag not present.
var loc = new Intl.Locale("sv-SE");
assert.sameValue(loc.baseName, "sv-SE");
assert.sameValue(loc.language, "sv");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, "SE");
assert.sameValue(loc.variants, undefined);

// 'variant' subtag present.
var loc = new Intl.Locale("de-1901");
assert.sameValue(loc.baseName, "de-1901");
assert.sameValue(loc.language, "de");
assert.sameValue(loc.script, undefined);
assert.sameValue(loc.region, undefined);
assert.sameValue(loc.variants, '1901');
