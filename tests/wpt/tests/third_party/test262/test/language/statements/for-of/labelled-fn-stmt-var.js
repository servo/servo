// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: It is a Syntax Error if IsLabelledFunction(Statement) is true.
negative:
  phase: parse
  type: SyntaxError
esid: sec-semantics-static-semantics-early-errors
es6id: 13.7.1.1
info: |
    Although Annex B describes an extension which permits labelled function
    declarations outside of strict mode, this early error is applied regardless
    of the language mode.
---*/

$DONOTEVALUATE();

for (var x of []) label1: label2: function f() {}
