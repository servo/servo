// Copyright (C) 2019 Bocoup. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Objects whose specified property is configurable satisfy the assertion.
includes: [propertyHelper.js]
---*/

Object.defineProperty(this, 'Object', {
  configurable: true,
  value: Object
});

verifyConfigurable(this, 'Object');
