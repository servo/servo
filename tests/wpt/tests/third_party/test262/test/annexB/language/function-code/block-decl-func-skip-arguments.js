// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Functions named 'arguments' have legacy hoisting semantics
esid: sec-web-compat-functiondeclarationinstantiation
flags: [noStrict]
info: |
    FunctionDeclarationInstantiation ( _func_, _argumentsList_ )

    [...]
    7. Let _parameterNames_ be the BoundNames of _formals_.
    [...]
    22. If argumentsObjectNeeded is true, then
      f. Append "arguments" to parameterNames.

    Changes to FunctionDeclarationInstantiation

    [...]
    ii. If replacing the |FunctionDeclaration| _f_ with a |VariableStatement| that has _F_
        as a |BindingIdentifier| would not produce any Early Errors for _func_ and _F_ is
        not an element of _parameterNames_, then
      [...]
---*/

// Simple parameters
(function() {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(arguments.toString(), "[object Arguments]");
}());

// Single named parameter
(function(x) {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(arguments.toString(), "[object Arguments]");
}());

// Non-simple parameters
(function(..._) {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(arguments.toString(), "[object Arguments]");
}());
