// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.collations
description: The return value does not contain invalid values
info: |
  10.2.3 Internal Slots
  - The values *"standard"* and *"search"* must not be used as elements in any
    [[SortLocaleData]].[[<locale>]].[[co]] and
    [[SearchLocaleData]].[[<locale>]].[[co]] List.
features: [Intl.Locale, Intl.Locale-info, Array.prototype.includes]
locale: [ar, de, en, ja, ko, sv, tr, zh]
---*/

var tags = ["ar", "de", "en", "ja", "ko", "sv", "tr", "zh"];

for (var i = 0; i < tags.length; i++) {
  var tag = tags[i];
  var collations = new Intl.Locale(tag).getCollations();
  assert.notSameValue(collations.length, 0,
    "getCollations() for " + tag + " has at least one element");
  for (var j = 0; j < collations.length; j++) {
    assert.notSameValue(collations[j], "standard",
      "getCollations() for " + tag + " must not contain 'standard'");
    assert.notSameValue(collations[j], "search",
      "getCollations() for " + tag + " must not contain 'search'");
  }
}
