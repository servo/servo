// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules
description: Intl.PluralRules.name is "PluralRules"
author: Zibi Braniecki
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.PluralRules, 'name', {
  value: 'PluralRules',
  writable: false,
  enumerable: false,
  configurable: true
});
