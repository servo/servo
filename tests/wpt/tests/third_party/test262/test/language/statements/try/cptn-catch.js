// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.15.8
description: Completion value from `catch` clause of a try..catch statement
info: |
    TryStatement : try Block Catch

    1. Let B be the result of evaluating Block.
    2. If B.[[type]] is throw, then
       a. Let C be CatchClauseEvaluation of Catch with parameter B.[[value]].
    3. Else B.[[type]] is not throw,
       [...]
    4. If C.[[type]] is return, or C.[[type]] is throw, return Completion(C).
    5. If C.[[value]] is not empty, return Completion(C).
    6. Return Completion{[[type]]: C.[[type]], [[value]]: undefined,
       [[target]]: C.[[target]]}.

    13.15.7 Runtime Semantics: CatchClauseEvaluation

    Catch : catch ( CatchParameter ) Block

    [...]
    7. Let B be the result of evaluating Block.
    8. Set the running execution context’s LexicalEnvironment to oldEnv.
    9. Return Completion(B).
---*/

assert.sameValue(eval('1; try { throw null; } catch (err) { }'), undefined);
assert.sameValue(eval('2; try { throw null; } catch (err) { 3; }'), 3);
