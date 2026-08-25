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

// |true && a| is constant-folded to |a|, ensure the destructuring assignment
// validation takes place before constant-folding.
for (let prefix of ["null,", "var", "let", "const"]) {
    assertSyntaxError(`${prefix} [true && a] = [];`);
    assertSyntaxError(`${prefix} {p: true && a} = {};`);
}

