// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createbuiltinfunction
description: NumberFormat bound format function property order
info: |
  Set order: "length", "name"
includes: [compareArray.js]
---*/

var formatFn = new Intl.NumberFormat().format;

assert.compareArray(
  Object.getOwnPropertyNames(formatFn),
  ['length', 'name']
);
