// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype
description: >
  The initial value of DataView.prototype is the DataView prototype object.
info: |
  The initial value of DataView.prototype is the intrinsic object
  %DataViewPrototype%.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(DataView, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
