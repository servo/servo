// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-labelled-statements-static-semantics-containsundefinedcontinuetarget
es6id: 13.13.4
description: Does not modify `iterationSet`
info: |
  With arguments iterationSet and labelSet.

  LabelledStatement : LabelIdentifier : LabelledItem

  1. Let label be the StringValue of LabelIdentifier.
  2. Let newLabelSet be a copy of labelSet with label appended.
  3. Return ContainsUndefinedContinueTarget of LabelledItem with arguments
     iterationSet and newLabelSet.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

do {
  test262: {
    continue test262;
  }
} while (false)
