// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
es6id: 15.1.1
description: >
  A direct eval in the functon code of a non-ArrowFunction may contain
  SuperProperty
info: |
  - It is a Syntax Error if StatementList Contains super unless the source code
    containing super is eval code that is being processed by a direct eval that
    is contained in function code that is not the function code of an
    ArrowFunction.
features: [super]
---*/

var superProp = null;
var o = {
  test262: null,
  method() {
    superProp = eval('super.test262;');
  }
};

o.method();

assert.sameValue(superProp, undefined);

Object.setPrototypeOf(o, { test262: 262 });

o.method();

assert.sameValue(superProp, 262);
