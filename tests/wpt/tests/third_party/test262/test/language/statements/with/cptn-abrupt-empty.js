// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-with-statement-runtime-semantics-evaluation
description: >
    Statement completion value when body returns an empty abrupt completion
info: |
    WithStatement : with ( Expression ) Statement

    [...]
    7. Let C be the result of evaluating Statement.
    8. Set the running execution context's LexicalEnvironment to oldEnv.
    9. Return Completion(UpdateEmpty(C, undefined)).
flags: [noStrict]
---*/

assert.sameValue(
  eval('1; do { 2; with({}) { 3; break; } 4; } while (false);'), 3
);
assert.sameValue(
  eval('5; do { 6; with({}) { break; } 7; } while (false);'), undefined
);

assert.sameValue(
  eval('8; do { 9; with({}) { 10; continue; } 11; } while (false)'), 10
);
assert.sameValue(
  eval('12; do { 13; with({}) { continue; } 14; } while (false)'), undefined
);
