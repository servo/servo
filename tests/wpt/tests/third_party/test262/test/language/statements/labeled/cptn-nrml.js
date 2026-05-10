// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-labelled-statements-runtime-semantics-labelledevaluation
es6id: 13.13.14
description: Completion value when LabelledItem returns normally
info: |
  LabelledStatement : LabelIdentifier : LabelledItem

  1. Let label be the StringValue of LabelIdentifier.
  2. Append label as an element of labelSet.
  3. Let stmtResult be LabelledEvaluation of LabelledItem with argument
     labelSet.
  4. If stmtResult.[[Type]] is break and SameValue(stmtResult.[[Target]],
     label) is true, then
     [...]
  5. Return Completion(stmtResult).
---*/

assert.sameValue(eval('test262id: 2;'), 2);
