// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.15.8
description: Completion value from `try` clause of a try..catch statement
info: |
    TryStatement : try Block Catch

    1. Let B be the result of evaluating Block.
    2. If B.[[type]] is throw, then
       [...]
    3. Else B.[[type]] is not throw,
       a. Let C be B.
    4. If C.[[type]] is return, or C.[[type]] is throw, return Completion(C).
    5. If C.[[value]] is not empty, return Completion(C).
    6. Return Completion{[[type]]: C.[[type]], [[value]]: undefined,
       [[target]]: C.[[target]]}.
---*/

assert.sameValue(eval('1; try { } catch (err) { }'), undefined);
assert.sameValue(eval('2; try { 3; } catch (err) { }'), 3);
assert.sameValue(eval('4; try { } catch (err) { 5; }'), undefined);
assert.sameValue(eval('6; try { 7; } catch (err) { 8; }'), 7);
