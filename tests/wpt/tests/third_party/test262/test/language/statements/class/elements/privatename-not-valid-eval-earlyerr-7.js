// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
description: Early error when referencing privatename on object, outside of class.
info: |
  Static Semantics: Early Errors
    ScriptBody : StatementList

    It is a Syntax Error if AllPrivateNamesValid of StatementList with an empty List as an argument is false unless the source code is eval code that is being processed by a direct eval.

features: [class-fields-private]
---*/

var executed = false;
var obj = {};

assert.throws(SyntaxError, function() {
  eval("executed = true; obj.#x;");
});

assert.sameValue(executed, false);
