// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.select
description: Intl.PluralRules.prototype.select.name is "select"
author: Zibi Braniecki
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.PluralRules.prototype.select, "name", {
  value: "select",
  writable: false,
  enumerable: false,
  configurable: true,
});
