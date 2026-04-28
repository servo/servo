// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
description: Early error when referencing privatename in constructor without being declared in class fields
info: |
  Static Semantics: Early Errors
    ScriptBody : StatementList

    It is a Syntax Error if AllPrivateNamesValid of StatementList with an empty List as an argument is false unless the source code is eval code that is being processed by a direct eval.

features: [class, class-fields-private]
---*/

var executed = false;

class C {
  constructor() {
    eval("executed = true; this.#x;");
  }
}

assert.throws(SyntaxError, function() {
  new C();
});

assert.sameValue(executed, false);
