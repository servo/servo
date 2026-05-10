// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of a null options argument to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
    3. Else
        a. Let options be ? ToObject(options).
features: [Intl.ListFormat]
---*/

assert.sameValue(typeof Intl.ListFormat, "function");
assert.throws(TypeError, function() {
  new Intl.ListFormat([], null);
});
