// Copyright (c) 2018 Leo Balter.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator-runtime-semantics-evaluation
description: >
  The delete expression should return true if the right hand UnaryExpression is not a Reference
info: |
  Runtime Semantics: Evaluation
    UnaryExpression : delete UnaryExpression

    1. Let ref be the result of evaluating UnaryExpression.
    2. ReturnIfAbrupt(ref).
    3. If Type(ref) is not Reference, return true.
---*/

var a = { b: 42 };
assert.sameValue(delete void a.b, true, 'delete void a.b');
assert.sameValue(delete void 0, true, 'delete void 0');
assert.sameValue(delete typeof 0, true, 'delete typeof 0');
assert.sameValue(delete delete 0, true, 'delete delete 0');
assert.sameValue(delete void typeof +-~!0, true, 'delete void typeof +-~!0');
assert.sameValue(delete {x:1}, true, 'delete {x:1}');
assert.sameValue(delete null, true, 'delete null');
assert.sameValue(delete true, true, 'delete true');
assert.sameValue(delete false, true, 'delete false');
assert.sameValue(delete 0, true, 'delete 0');
assert.sameValue(delete 1, true, 'delete 1');
assert.sameValue(delete '', true, 'delete ""');
assert.sameValue(delete 'Test262', true, 'delete "Test262"');
assert.sameValue(delete 'Test262'[100], true, 'delete "Test262"[100]');
assert.sameValue(delete typeof +-~!0, true, 'delete typeof +-~!0');
assert.sameValue(delete +-~!0, true, 'delete +-~!0');
assert.sameValue(delete -~!0, true, 'delete -~!0');
assert.sameValue(delete ~!0, true, 'delete ~!0');
assert.sameValue(delete !0, true, 'delete !0');
