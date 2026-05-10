// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules
description: Intl.PluralRules.length.
author: Zibi Braniecki
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.PluralRules, 'length', {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
