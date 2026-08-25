// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.15.8
description: Completion value from `finally` clause of a try..finally statement
info: |
    TryStatement : try Block Finally

    1. Let B be the result of evaluating Block.
    2. Let F be the result of evaluating Finally.
    3. If F.[[type]] is normal, let F be B.
    4. If F.[[type]] is return, or F.[[type]] is throw, return Completion(F).
    5. If F.[[value]] is not empty, return Completion(F).
    6. Return Completion{[[type]]: F.[[type]], [[value]]: undefined,
       [[target]]: F.[[target]]}.
---*/


assert.sameValue(eval('1; try { } finally { }'), undefined);
assert.sameValue(eval('2; try { 3; } finally { }'), 3);
assert.sameValue(eval('4; try { } finally { 5; }'), undefined);
assert.sameValue(eval('6; try { 7; } finally { 8; }'), 7);
