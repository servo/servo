// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.resolvedOptions.name
description: Intl.PluralRules.resolvedOptions.name is "resolvedOptions"
author: Zibi Braniecki
includes: [propertyHelper.js]
---*/

verifyProperty(Intl.PluralRules.prototype.resolvedOptions, "name", {
  value: "resolvedOptions",
  writable: false,
  enumerable: false,
  configurable: true,
});
