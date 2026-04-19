// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
es6id: 15.1.1
description: An ArrowFunction in global code may not contain `new.target`
info: |
  - It is a Syntax Error if StatementList Contains NewTarget unless the source
    code containing NewTarget is eval code that is being processed by a direct
    eval that is contained in function code that is not the function code of an
    ArrowFunction.

  14.2.3 Static Semantics: Contains

  With parameter symbol.

  ArrowFunction : ArrowParameters => ConciseBody

  1. If symbol is not one of NewTarget, SuperProperty, SuperCall, super or
     this, return false.
  2. If ArrowParameters Contains symbol is true, return true.
  3. Return ConciseBody Contains symbol.

  NOTE Normally, Contains does not look inside most function forms. However,
       Contains is used to detect new.target, this, and super usage within an
       ArrowFunction.
features: [arrow-function, new.target]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

() => {
  new.target;
};
