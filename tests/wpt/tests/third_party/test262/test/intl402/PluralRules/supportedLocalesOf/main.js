// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.supportedLocalesOf
description: >
    Tests that Intl.PluralRules has a supportedLocalesOf property, and
    it works as planned.
author: Zibi Braniecki
---*/

var defaultLocale = new Intl.PluralRules().resolvedOptions().locale;
var notSupported = 'zxx'; // "no linguistic content"
var requestedLocales = [defaultLocale, notSupported];
    
var supportedLocales;

assert(Intl.PluralRules.hasOwnProperty('supportedLocalesOf'), "Intl.PluralRules doesn't have a supportedLocalesOf property.");
    
supportedLocales = Intl.PluralRules.supportedLocalesOf(requestedLocales);
assert.sameValue(supportedLocales.length, 1, 'The length of supported locales list is not 1.');

assert.sameValue(supportedLocales[0], defaultLocale, 'The default locale is not returned in the supported list.');
