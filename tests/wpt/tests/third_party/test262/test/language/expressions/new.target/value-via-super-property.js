// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword-runtime-semantics-evaluation
es6id: 12.3.5.1
description: Value when invoked via SuperCall
info: |
  SuperCall : super Arguments

  1. Let newTarget be GetNewTarget().
  [...]
  6. Let result be ? Construct(func, argList, newTarget).
  [...]
features: [class, new.target]
---*/

var newTarget = null;

class Parent {
  get attr() {
    newTarget = new.target;
  }
}

class Child extends Parent {
  constructor() {
    super();
    super.attr;
  }
}

new Child();

assert.sameValue(newTarget, undefined);
