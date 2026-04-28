// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-calls-runtime-semantics-evaluation
es6id: 12.3.4.1
description: Correct retrieval of environment's "with" base object
info: |
  4. If Type(ref) is Reference, then
     a. If IsPropertyReference(ref) is true, then
        [...]
     b. Else the base of ref is an Environment Record,
        i. Let refEnv be GetBase(ref).
        ii. Let thisValue be refEnv.WithBaseObject().
  [...]
  8. Return ? EvaluateDirectCall(func, thisValue, Arguments, tailCall).
flags: [noStrict]
---*/

var viaMember, viaCall;
var obj = {
  method: function() {
    viaCall = this;
  },
  get attribute() {
    viaMember = this;
  }
};

with (obj) {
  method();
  attribute;
}

assert.sameValue(viaCall, obj, 'via CallExpression');
assert.sameValue(viaMember, obj, 'via MemberExpression');
