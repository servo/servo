// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

var AsyncGenerator = async function*(){}.constructor;

function assertSyntaxError(code) {
    var functionCode = `async function* f() { ${code} }`;
    assert.throws(SyntaxError, () => AsyncGenerator(code), "AsyncGenerator:" + code);
    assert.throws(SyntaxError, () => eval(functionCode), "eval:" + functionCode);
    var ieval = eval;
    assert.throws(SyntaxError, () => ieval(functionCode), "indirect eval:" + functionCode);
}

assertSyntaxError(`for await (;;) ;`);

for (var decl of ["", "var", "let", "const"]) {
    for (var head of ["a", "a = 0", "a, b", "[a]", "[a] = 0", "{a}", "{a} = 0"]) {
        // Ends with C-style for loop syntax.
        assertSyntaxError(`for await (${decl} ${head} ;;) ;`);

        // Ends with for-in loop syntax.
        assertSyntaxError(`for await (${decl} ${head} in null) ;`);
    }
}
