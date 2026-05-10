// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Objects whose specified property is configurable satisfy the assertion.
includes: [propertyHelper.js]
---*/

var obj = {};
Object.defineProperty(obj, 'a', {
  configurable: true
});

verifyConfigurable(obj, 'a');
