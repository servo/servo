// Copyright 2024 Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules.prototype.resolvedoptions
description: >
  Tests that Intl.PluralRules.prototype.resolvedOptions elements given in correct order.
info: |
  Intl.PluralRules.prototype.resolvedOptions ()

  4. Let pluralCategories be a List of Strings containing all possible results of PluralRuleSelect for the selected locale pr.[[Locale]], sorted according to the following order: "zero", "one", "two", "few", "many", "other".

includes: [compareArray.js]
locale: [ar, en, fa, fr, gv, ko, sl]
---*/

assert.compareArray(new Intl.PluralRules('ar').resolvedOptions().pluralCategories, ['zero', 'one', 'two', 'few', 'many', 'other'], "pluralCategories order or contents incorrect for 'ar' locale");
assert.compareArray(new Intl.PluralRules('en').resolvedOptions().pluralCategories, ['one', 'other'], "pluralCategories order or contents incorrect for 'en' locale");
assert.compareArray(new Intl.PluralRules('fa').resolvedOptions().pluralCategories, ['one', 'other'], "pluralCategories order or contents incorrect for 'fa' locale");
assert.compareArray(new Intl.PluralRules('fr').resolvedOptions().pluralCategories, ['one', 'many', 'other'], "pluralCategories order or contents incorrect for 'fr' locale");
assert.compareArray(new Intl.PluralRules('gv').resolvedOptions().pluralCategories, ['one', 'two', 'few', 'many', 'other'], "pluralCategories order or contents incorrect for 'gv' locale");
assert.compareArray(new Intl.PluralRules('ko').resolvedOptions().pluralCategories, ['other'], "pluralCategories order or contents incorrect for 'ko' locale");
assert.compareArray(new Intl.PluralRules('sl').resolvedOptions().pluralCategories, ['one', 'two', 'few', 'other'], "pluralCategories order or contents incorrect for 'sl' locale");
