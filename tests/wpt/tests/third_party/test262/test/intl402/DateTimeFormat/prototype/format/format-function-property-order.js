// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createbuiltinfunction
description: DateTimeFormat bound format function property order
info: |
  Set order: "length", "name"
includes: [compareArray.js]
---*/

var formatFn = new Intl.DateTimeFormat().format;

assert.compareArray(
  Object.getOwnPropertyNames(formatFn),
  ['length', 'name']
);
