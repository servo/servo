// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: using with null or undefined should not throw.
includes: [compareArray.js]
features: [explicit-resource-management]
---*/

// Use using with null does not throw --------------
let withNullvalues = [];

(function TestUsingWithNull() {
  using x = null;
  withNullvalues.push(42);
})();
assert.compareArray(withNullvalues, [42]);

// Use using with undefined does not throw --------------
let withUndefinedvalues = [];

(function TestUsingWithUndefined() {
  using x = undefined;
  withUndefinedvalues.push(42);
})();
assert.compareArray(withUndefinedvalues, [42]);
