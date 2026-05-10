// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: Attempts to delete super reference property references throws ReferenceError exception
features: [class]
---*/

class X {
  method() {
    return this;
  }
}

class Y extends X {
  method() {
    delete super.method;
  }
}

const y = new Y();

assert.throws(ReferenceError, () => {
  y.method();
});
