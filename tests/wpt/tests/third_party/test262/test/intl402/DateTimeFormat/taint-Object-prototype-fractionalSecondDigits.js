// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Tests that the behavior of a Record is not affected by
    adversarial  changes to Object.prototype.
includes: [testIntl.js]
features: [Intl.DateTimeFormat-fractionalSecondDigits]
---*/

taintProperties(["fractionalSecondDigits"]);

var locale = new Intl.DateTimeFormat(undefined, {localeMatcher: "lookup"}).resolvedOptions().locale;
assert(isCanonicalizedStructurallyValidLanguageTag(locale), "DateTimeFormat returns invalid locale " + locale + ".");
