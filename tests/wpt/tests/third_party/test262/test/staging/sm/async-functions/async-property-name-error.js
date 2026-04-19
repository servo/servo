// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

function assertSyntaxError(code) {
    assert.throws(SyntaxError, () => { Function(code); }, "Function:" + code);
    assert.throws(SyntaxError, () => { eval(code); }, "eval:" + code);
    var ieval = eval;
    assert.throws(SyntaxError, () => { ieval(code); }, "indirect eval:" + code);
}

assertSyntaxError(`({async async: 0})`);
assertSyntaxError(`({async async})`);
assertSyntaxError(`({async async, })`);
assertSyntaxError(`({async async = 0} = {})`);

for (let decl of ["var", "let", "const"]) {
    assertSyntaxError(`${decl} {async async: a} = {}`);
    assertSyntaxError(`${decl} {async async} = {}`);
    assertSyntaxError(`${decl} {async async, } = {}`);
    assertSyntaxError(`${decl} {async async = 0} = {}`);
}
