// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCollations
description: The returned array is sorted in lexicographic code unit order
info: |
  CollationsOfLocale ( loc )

  5. Let _sorted_ be a copy of _list_, sorted according to lexicographic code
     unit order.
  6. Return CreateArrayFromList(_sorted_).
features: [Intl.Locale,Intl.Locale-info]
locale: [ar, de, en, ja, ko, sv, tr, zh]
---*/

var tags = ["ar", "de", "en", "ja", "ko", "sv", "tr", "zh"];

for (var i = 0; i < tags.length; i++) {
  var tag = tags[i];
  var collations = new Intl.Locale(tag).getCollations();
  var sortedCollations = [].concat(new Intl.Locale(tag).getCollations()).sort();
  assert.compareArray(
    collations,
    sortedCollations,
    "getCollations() for " + tag + " should be sorted in lexicographic code unit order"
  );
}
