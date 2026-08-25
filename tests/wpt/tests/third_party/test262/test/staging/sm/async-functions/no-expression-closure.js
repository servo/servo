// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

function assertSyntaxError(code) {
    assert.throws(SyntaxError, function () { Function(code); }, "Function:" + code);
    assert.throws(SyntaxError, function () { eval(code); }, "eval:" + code);
    var ieval = eval;
    assert.throws(SyntaxError, function () { ieval(code); }, "indirect eval:" + code);
}

// AsyncFunction statement
assertSyntaxError(`async function f() 0`);

// AsyncFunction expression
assertSyntaxError(`void async function() 0`);
assertSyntaxError(`void async function f() 0`);
