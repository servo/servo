// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_19
description: >
    Tests that the currency style can not be used without a specified
    currency.
author: Norbert Lindenberg
---*/

var defaultLocale = new Intl.NumberFormat().resolvedOptions().locale;

assert.throws(TypeError, function () {
        return new Intl.NumberFormat([defaultLocale], {style: "currency"});
}, "Throws TypeError when currency code is not specified.");

assert.throws(TypeError, function () {
        return new Intl.NumberFormat([defaultLocale + "-u-cu-krw"], {style: "currency"});
}, "Throws TypeError when currency code is not specified; Currenty code from Unicode locale extension sequence is ignored.");
