// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-intl-pluralrules-constructor
description: Tests that Intl.PluralRules can be subclassed.
author: Zibi Braniecki
includes: [compareArray.js]
---*/

// get a plural-rules and have it format an array of dates for comparison with the subclass
var locales = ["tlh", "id", "en"];
var a = [1, 5, 12];

var referencePluralRules = new Intl.PluralRules(locales);
var referenceSelected = a.map(referencePluralRules.select.bind(referencePluralRules));

class MyPluralRules extends Intl.PluralRules {
  constructor(locales, options) {
    super(locales, options);
    // could initialize MyPluralRules properties
  }
  // could add methods to MyPluralRules.prototype
}

var pr = new MyPluralRules(locales);
var actual = a.map(pr.select.bind(pr));
assert.compareArray(actual, referenceSelected);
