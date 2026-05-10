// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.15.8
description: >
    Completion value from `finally` clause of a try..catch..finally statement
    (when `catch` block is not executed)
info: |
    TryStatement : try Block Catch Finally

    1. Let B be the result of evaluating Block.
    2. If B.[[type]] is throw, then
       [...]
    3. Else B.[[type]] is not throw, let C be B.
    4. Let F be the result of evaluating Finally.
    5. If F.[[type]] is normal, let F be C.
    6. If F.[[type]] is return, or F.[[type]] is throw, return Completion(F).
    7. If F.[[value]] is not empty, return NormalCompletion(F.[[value]]).
    8. Return Completion{[[type]]: F.[[type]], [[value]]: undefined,
       [[target]]: F.[[target]]}.
---*/

assert.sameValue(eval('1; try { } catch (err) { } finally { }'), undefined);
assert.sameValue(eval('2; try { } catch (err) { 3; } finally { }'), undefined);
assert.sameValue(eval('4; try { } catch (err) { } finally { 5; }'), undefined);
assert.sameValue(eval('6; try { } catch (err) { 7; } finally { 8; }'), undefined);
assert.sameValue(eval('9; try { 10; } catch (err) { } finally { }'), 10);
assert.sameValue(eval('11; try { 12; } catch (err) { 13; } finally { }'), 12);
assert.sameValue(eval('14; try { 15; } catch (err) { } finally { 16; }'), 15);
assert.sameValue(eval('17; try { 18; } catch (err) { 19; } finally { 20; }'), 18);
