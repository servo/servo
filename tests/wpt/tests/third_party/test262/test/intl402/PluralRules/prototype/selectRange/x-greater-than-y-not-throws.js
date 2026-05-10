// Copyright 2022 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRang
description: >
  "selectRange" basic tests when argument  x > y, return a string.
info: |
  1.1.6 ResolvePluralRange ( pluralRules, x, y )
features: [Intl.NumberFormat-v3]
---*/

const pr = new Intl.PluralRules();

// 1. If x > y, return a string.
assert.sameValue(typeof pr.selectRange(201, 102), "string", "should not throw RangeError");
