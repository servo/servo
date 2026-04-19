// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createbuiltinfunction
description: Collator bound compare function property order
info: |
  Set order: "length", "name"
includes: [compareArray.js]
---*/

var compareFn = new Intl.Collator().compare;

assert.compareArray(
  Object.getOwnPropertyNames(compareFn),
  ['length', 'name']
);
