// Copyright 2018 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.supportedLocalesOf
description: Tests that Intl.Segmenter has a supportedLocalesOf property, and it works as expected.
features: [Intl.Segmenter]
---*/

assert.sameValue(typeof Intl.Segmenter.supportedLocalesOf, "function",
                 "supportedLocalesOf should be supported.");

const defaultLocale = new Intl.Segmenter().resolvedOptions().locale;
const notSupported = "zxx"; // "no linguistic content"
const requestedLocales = [defaultLocale, notSupported];

const supportedLocales = Intl.Segmenter.supportedLocalesOf(requestedLocales);
assert.sameValue(supportedLocales.length, 1, "The length of supported locales list is not 1.");
assert.sameValue(supportedLocales[0], defaultLocale, "The default locale is not returned in the supported list.");
