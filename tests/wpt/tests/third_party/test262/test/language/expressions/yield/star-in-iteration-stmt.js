// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions
es6id: 14.4
description: >
  YieldExpression operand may not include the `in` keyword in contexts where it
  is disallowed
info: |
  Syntax

  yield [no LineTerminator here] * AssignmentExpression[?In, +Yield]
negative:
  phase: parse
  type: SyntaxError
features: [generators]
---*/

$DONOTEVALUATE();

function* g() {
  for (yield * '' in {}; ; ) ;
}
