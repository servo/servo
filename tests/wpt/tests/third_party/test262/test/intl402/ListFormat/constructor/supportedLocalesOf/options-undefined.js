// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.supportedLocalesOf
description: Checks handling of an undefined options argument to the supportedLocalesOf function.
info: |
    SupportedLocales ( availableLocales, requestedLocales, options )

    1. If options is not undefined, then
        b. Let matcher be ? GetOption(options, "localeMatcher", "string", «"lookup", "best fit"», "best fit").
features: [Intl.ListFormat]
---*/

assert.sameValue(typeof Intl.ListFormat.supportedLocalesOf, "function",
                 "Should support Intl.ListFormat.supportedLocalesOf.");

Object.defineProperties(Object.prototype, {
  "localeMatcher": {
    get() { throw new Error("Should not call localeMatcher getter"); }
  }
});

assert.sameValue(Array.isArray(Intl.ListFormat.supportedLocalesOf()), true, "No arguments");
assert.sameValue(Array.isArray(Intl.ListFormat.supportedLocalesOf([])), true, "One argument");
assert.sameValue(Array.isArray(Intl.ListFormat.supportedLocalesOf([], undefined)), true, "Two arguments");
