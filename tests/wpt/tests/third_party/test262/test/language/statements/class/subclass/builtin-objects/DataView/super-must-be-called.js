// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 24.2.2
description: Super need to be called to initialize internals
info: |
  24.2.2 The DataView Constructor

  ...

  The DataView constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified DataView behaviour must include a super call
  to the DataView constructor to create and initialize subclass instances with
  the internal state necessary to support the DataView.prototype built-in
  methods.
---*/

class DV1 extends DataView {
  constructor() {}
}

var buffer = new ArrayBuffer(1);

assert.throws(ReferenceError, function() {
  new DV1(buffer);
});

class DV2 extends DataView {
  constructor(length) {
    super(length);
  }
}

new DV2(buffer);
