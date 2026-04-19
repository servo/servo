// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
description: Early error when referencing privatename in function in class without declaring in field
info: |
  Static Semantics: Early Errors
    ScriptBody : StatementList

    It is a Syntax Error if AllPrivateNamesValid of StatementList with an empty List as an argument is false unless the source code is eval code that is being processed by a direct eval.

features: [class, class-fields-private]
---*/

var executed = false;

class C {
  f() {
    eval("executed = true; this.#x;");
  }
}

assert.throws(SyntaxError, function() {
  new C().f();
});

assert.sameValue(executed, false);
