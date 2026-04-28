// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Referencing the arguments object from a default parameter (method)
esid: sec-runtime-semantics-definemethod
es6id: 14.3.8
features: [default-parameters]
info: |
    MethodDefinition : PropertyName ( StrictFormalParameters ) { FunctionBody }

    [...]
    6. Let closure be FunctionCreate(kind, StrictFormalParameters,
       FunctionBody, scope, strict). If functionPrototype was passed as a
       parameter then pass its value as the functionPrototype optional argument
       of FunctionCreate.
    [...]

    9.2.1 [[Call]] ( thisArgument, argumentsList)

    [...]
    7. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    [...]

    9.2.1.3 OrdinaryCallEvaluateBody ( F, argumentsList )

    1. Let status be FunctionDeclarationInstantiation(F, argumentsList).
    [...]

    9.2.12 FunctionDeclarationInstantiation(func, argumentsList)

    [...]
    23. Let iteratorRecord be Record {[[iterator]]:
        CreateListIterator(argumentsList), [[done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        b. Let formalStatus be IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
    [...]

    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments. 
---*/

var callCount = 0;
var obj = {
  method(x, _ = 0) {
    assert.sameValue(x, undefined, 'parameter binding value (initial)');
    assert.sameValue(
      arguments[0], undefined, 'arguments property value (initial)'
    );

    arguments[0] = 1;

    assert.sameValue(
      x, undefined, 'parameter binding value (after arguments modification)'
    );
    assert.sameValue(
      arguments[0], 1, 'arguments property value (after arguments modification)'
    );

    x = 2;

    assert.sameValue(
      x, 2, 'parameter binding value (after parameter binding modification)'
    );
    assert.sameValue(
      arguments[0],
      1,
      'arguments property value (after parameter binding modification)'
    );
    callCount = callCount + 1;
  }
};

obj.method();

assert.sameValue(callCount, 1, 'method invoked exactly once');
