// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
es6id: 14.4.14
description: GetValue invoked on Reference value
info: |
  YieldExpression : yield AssignmentExpression

  1. Let exprRef be the result of evaluating AssignmentExpression.
  2. Let value be ? GetValue(exprRef).
features: [generators]
---*/

var err;
function* g() {
  try {
    yield test262unresolvable;
  } catch (_err) {
    err = _err;
  }
}
var iter = g();
var result;

result = iter.next();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
assert.sameValue(typeof err, 'object');
assert.sameValue(err.constructor, ReferenceError);
