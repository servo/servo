// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-constructor
description: >
  The DataView constructor is extensible
info: |
  17 ECMAScript Standard Built-in Objects

  Unless specified otherwise, the [[Extensible]] internal slot of a built-in
  object initially has the value true.
---*/

assert.sameValue(Object.isExtensible(DataView), true);
