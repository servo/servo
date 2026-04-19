// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
    ShadowRealm.prototype.evaluate is an ECMAScript Standard built-in object function.
includes: [propertyHelper.js]
features: [ShadowRealm]
---*/

verifyProperty(ShadowRealm.prototype, "evaluate", {
  enumerable: false,
  writable: true,
  configurable: true,
});
