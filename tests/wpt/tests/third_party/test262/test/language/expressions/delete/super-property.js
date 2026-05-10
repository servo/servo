// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: SuperReferences may not be deleted
info: |
  [...]
  5.If IsPropertyReference(ref) is true, then
    a. If IsSuperReference(ref) is true, throw a ReferenceError exception.
features: [class]
---*/

class C extends Object {
  constructor() {
    super();
    delete super.x;
  }
}

assert.throws(ReferenceError, () => {
  new C();
});
