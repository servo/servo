// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-try-statement-runtime-semantics-evaluation
description: >
  Direct eval try/catch/finally for completion value
info: |
  TryStatement : try Block Catch Finally

    Let B be the result of evaluating Block.
    If B.[[Type]] is throw, let C be CatchClauseEvaluation of Catch with argument B.[[Value]].
    Else, let C be B.
    Let F be the result of evaluating Finally.
    If F.[[Type]] is normal, set F to C.
    Return Completion(UpdateEmpty(F, undefined)).
---*/

assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { 42; break; -2 }; } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1; } finally { 42; break; -3 }; } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { break; -2 }; } while (false);'),
  undefined
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1; } finally { break; -3 }; } while (false);'),
  undefined
);
assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { 42; break; -3 }; -77 } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1; } finally { 42; break; -3 }; -77 } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { break; -3 }; -77 } while (false);'),
  undefined
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1; } finally { break; -3 }; -77 } while (false);'),
  undefined
);
assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { 42; continue; -3 }; } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1; } finally { 42; continue; -3 }; } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { 39 } catch (e) { -1 } finally { 42; continue; -3 }; -77 } while (false);'),
  42
);
assert.sameValue(
  eval('99; do { -99; try { [].x.x } catch (e) { -1 } finally { 42; continue; -3 }; -77 } while (false);'),
  42
);
