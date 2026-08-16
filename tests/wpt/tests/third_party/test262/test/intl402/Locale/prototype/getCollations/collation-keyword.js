// Copyright 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getCollations
description: >
  u-co extension keyword and collation option override language lookup
info: |
  CollationsOfLocale ( loc )

  1. If _loc_.[[Collation]] is not *undefined*, then
    a. Return CreateArrayFromList(« _loc_.[[Collation]] »).
features: [Intl.Locale,Intl.Locale-info]
locale: [en, de, zh]
---*/

var testCases = [
  ["en", "phonebk"],
  ["de", "phonebk"],
  ["zh", "stroke"],
  ["und", "pinyin"],
  ["und", "emoji"]
];

for (var i = 0; i < testCases.length; i++) {
  var baseName = testCases[i][0];
  var collation = testCases[i][1];
  var fullTag = baseName + "-u-co-" + collation;

  assert.compareArray(
    new Intl.Locale(fullTag).getCollations(),
    [collation],
    "getCollations() for " + fullTag + " returns only " + collation
  );
  assert.compareArray(
    new Intl.Locale(baseName, { collation: collation }).getCollations(),
    [collation],
    "getCollations() for " + baseName + " with { collation: " + collation + " } returns only " + collation
  );
}
