// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat
description: Tests that the option roundingPriority is processed correctly.
features: [Intl.NumberFormat-v3]
includes: [testIntl.js]
---*/

testOption(
  Intl.NumberFormat,
  "roundingPriority",
  "string",
  ["auto", "morePrecision", "lessPrecision"],
  "auto"
);
