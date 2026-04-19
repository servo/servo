// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-constructor
description: >
  The DataView constructor is the %DataView% intrinsic object and the initial
  value of the DataView property of the global object.
---*/

assert.sameValue(typeof DataView, "function", "`typeof DataView` is `'function'`");
