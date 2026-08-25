// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.supportedLocalesOf
description: Checks handling of a null options argument to the supportedLocalesOf function.
info: |
    SupportedLocales ( availableLocales, requestedLocales, options )

    1. If options is not undefined, then
        a. Let options be ? ToObject(options).
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat.supportedLocalesOf, "function",
                 "Should support Intl.RelativeTimeFormat.supportedLocalesOf.");

assert.throws(TypeError, function() {
  Intl.RelativeTimeFormat.supportedLocalesOf([], null);
}, "Should throw when passing null as the options argument");
