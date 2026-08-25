// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 15.2.1.1
description: >
    It is a Syntax Error if ContainsUndefinedContinueTarget of ModuleItemList
    with arguments « » and « » is true.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

while (false) {
  continue undef;
}
