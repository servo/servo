// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
description: >
  Initializer is not evaluated when argument is an object with
  [[IsHTMLDDA]] internal slot.
info: |
  FunctionDeclaration :
    function BindingIdentifier ( FormalParameters ) { FunctionBody }

    [...]
    3. Let F be FunctionCreate(Normal, FormalParameters, FunctionBody,
    scope, strict).
    [...]

  [[Call]] ( thisArgument, argumentsList)

  [...]
  7. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
  [...]

  OrdinaryCallEvaluateBody ( F, argumentsList )

  1. Let status be FunctionDeclarationInstantiation(F, argumentsList).
  [...]

  FunctionDeclarationInstantiation(func, argumentsList)

  [...]
  23. Let iteratorRecord be Record {[[iterator]]:
  CreateListIterator(argumentsList), [[done]]: false}.
  24. If hasDuplicates is true, then
    [...]
  25. Else,
    b. Let formalStatus be IteratorBindingInitialization for formals with
    iteratorRecord and env as arguments.
  [...]

  Runtime Semantics: IteratorBindingInitialization

  FormalsList : FormalsList , FormalParameter

  [...]
  23. Let iteratorRecord be Record {[[Iterator]]:
  CreateListIterator(argumentsList), [[Done]]: false}.
  24. If hasDuplicates is true, then
    [...]
  25. Else,
    a. Perform ? IteratorBindingInitialization for formals with
    iteratorRecord and env as arguments.
  [...]
features: [default-parameters, IsHTMLDDA]
---*/

let initCount = 0;
const counter = function() {
  initCount += 1;
};

const arrow = (x = counter()) => x;
const IsHTMLDDA = $262.IsHTMLDDA;

assert.sameValue(arrow(IsHTMLDDA), IsHTMLDDA);
assert.sameValue(initCount, 0);

function fn(x, y = counter()) {
  return y;
}

assert.sameValue(fn(1, IsHTMLDDA), IsHTMLDDA);
assert.sameValue(initCount, 0);
