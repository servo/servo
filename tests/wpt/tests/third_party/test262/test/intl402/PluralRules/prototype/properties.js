// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-properties-of-intl-pluralrules-prototype-object
description: Tests that Intl.PluralRules.prototype has the required attributes.
author: Zibi Braniecki
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.PluralRules, "prototype", {
    writable: false,
    enumerable: false,
    configurable: false,
});
