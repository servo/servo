// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.2.2_a
description: >
    Tests that Intl.NumberFormat has a supportedLocalesOf  property,
    and it works as planned.
author: Roozbeh Pournader
---*/

var defaultLocale = new Intl.NumberFormat().resolvedOptions().locale;
var notSupported = 'zxx'; // "no linguistic content"
var requestedLocales = [defaultLocale, notSupported];
    
var supportedLocales;

assert(Intl.NumberFormat.hasOwnProperty('supportedLocalesOf'), "Intl.NumberFormat doesn't have a supportedLocalesOf property.");
    
supportedLocales = Intl.NumberFormat.supportedLocalesOf(requestedLocales);
assert.sameValue(supportedLocales.length, 1, 'The length of supported locales list is not 1.');
    
assert.sameValue(supportedLocales[0], defaultLocale, 'The default locale is not returned in the supported list.');
