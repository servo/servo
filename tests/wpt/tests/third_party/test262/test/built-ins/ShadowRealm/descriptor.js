// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm-constructor
description: >
  The ShadowRealm constructor is the initial value of the "ShadowRealm" property of the global object.
includes: [propertyHelper.js]
features: [ShadowRealm]
---*/

verifyProperty(this, "ShadowRealm", {
  enumerable: false,
  writable: true,
  configurable: true,
});
