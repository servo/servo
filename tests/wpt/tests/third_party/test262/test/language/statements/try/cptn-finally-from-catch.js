// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.15.8
description: >
    Completion value from `finally` clause of a try..catch..finally statement
    (following execution of `catch` block)
info: |
    TryStatement : try Block Catch Finally

    1. Let B be the result of evaluating Block.
    2. If B.[[type]] is throw, then
       a. Let C be CatchClauseEvaluation of Catch with parameter B.[[value]].
    [...]
    4. Let F be the result of evaluating Finally.
    5. If F.[[type]] is normal, let F be C.
    6. If F.[[type]] is return, or F.[[type]] is throw, return Completion(F).
    7. If F.[[value]] is not empty, return NormalCompletion(F.[[value]]).
    8. Return Completion{[[type]]: F.[[type]], [[value]]: undefined,
       [[target]]: F.[[target]]}.

    13.15.7 Runtime Semantics: CatchClauseEvaluation

    Catch : catch ( CatchParameter ) Block

    [...]
    7. Let B be the result of evaluating Block.
    8. Set the running execution context’s LexicalEnvironment to oldEnv.
    9. Return Completion(B).
---*/

assert.sameValue(
  eval('1; try { throw null; } catch (err) { } finally { }'), undefined
);
assert.sameValue(
  eval('2; try { throw null; } catch (err) { 3; } finally { }'), 3
);
assert.sameValue(
  eval('4; try { throw null; } catch (err) { } finally { 5; }'), undefined
);
assert.sameValue(
  eval('6; try { throw null; } catch (err) { 7; } finally { 8; }'), 7
);
