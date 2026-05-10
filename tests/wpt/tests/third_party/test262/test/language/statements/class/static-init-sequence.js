// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classelementevaluation
description: Static blocks are evaluated in the order they appear in the source text, interleaved with static fields
info: |
  5.1.14 Runtime Semantics: ClassDefinitionEvaluation

  [...]
  34. For each element elementRecord of staticElements in List order, do
    a. If elementRecord is a ClassFieldDefinition Record, then
        i. Let status be the result of performing DefineField(F,
        elementRecord).
    b. Else,
        i. Assert: fieldRecord is a ClassStaticBlockDefinition Record.
        ii. Let status be the result of performing EvaluateStaticBlock(F,
            elementRecord).
    [...]
features: [class-static-fields-public, class-static-block]
---*/

var sequence = [];

class C {
  static x = sequence.push('first field');
  static {
    sequence.push('first block');
  }
  static x = sequence.push('second field');
  static {
    sequence.push('second block');
  }
}

assert.sameValue(sequence[0], 'first field');
assert.sameValue(sequence[1], 'first block');
assert.sameValue(sequence[2], 'second field');
assert.sameValue(sequence[3], 'second block');
