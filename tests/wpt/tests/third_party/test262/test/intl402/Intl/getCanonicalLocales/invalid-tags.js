// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Throws a RangeError if the language tag is invalid.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
    1. Let ll be ? CanonicalizeLocaleList(locales).
    ...

  9.2.1 CanonicalizeLocaleList (locales)
    ...
    7. Repeat, while k < len
      ...
      c. If kPresent is true, then
        ...
        iv. If IsStructurallyValidLanguageTag(tag) is false, throw a RangeError exception.
        ...
includes: [testIntl.js]
---*/

var invalidLanguageTags = getInvalidLanguageTags();
for (var i = 0; i < invalidLanguageTags.length; ++i) {
  var invalidTag = invalidLanguageTags[i];
  assert.throws(RangeError, function() {
    Intl.getCanonicalLocales(invalidTag)
  }, "Language tag: " + invalidTag);
}
