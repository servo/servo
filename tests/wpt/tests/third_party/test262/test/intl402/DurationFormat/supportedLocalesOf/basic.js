// Copyright 2022 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.supportedLocalesOf
description: Tests that Intl.DurationFormat has a supportedLocalesOf property, and it works as expected.
features: [Intl.DurationFormat]
---*/

assert.sameValue(typeof Intl.DurationFormat.supportedLocalesOf, "function",
                 "supportedLocalesOf should be supported.");

const defaultLocale = new Intl.DurationFormat().resolvedOptions().locale;
const notSupported = "zxx"; // "no linguistic content"
const requestedLocales = [defaultLocale, notSupported];

const supportedLocales = Intl.DurationFormat.supportedLocalesOf(requestedLocales);
assert.sameValue(supportedLocales.length, 1, "The length of the supported locales list should be 1");
assert.sameValue(supportedLocales[0], defaultLocale, "The default locale is returned in the supported list.");
