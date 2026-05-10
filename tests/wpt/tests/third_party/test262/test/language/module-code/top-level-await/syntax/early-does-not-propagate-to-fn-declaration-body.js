// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  The Await capability does not propagate to the body of a function declaration
info: |
  ModuleItem:
    StatementListItem[~Yield, +Await, ~Return]

  StatementListItem[Yield, Await, Return]:
    Statement[?Yield, ?Await, ?Return]
    Declaration[?Yield, ?Await]

  Declaration[Yield, Await]:
    HoistableDeclaration[?Yield, ?Await, ~Default]
    ClassDeclaration[?Yield, ?Await, ~Default]
    LexicalDeclaration[+In, ?Yield, ?Await]

  HoistableDeclaration[Yield, Await, Default]:
    FunctionDeclaration[?Yield, ?Await, ?Default]
    GeneratorDeclaration[?Yield, ?Await, ?Default]
    AsyncFunctionDeclaration[?Yield, ?Await, ?Default]
    AsyncGeneratorDeclaration[?Yield, ?Await, ?Default]

  FunctionDeclaration[Yield, Await, Default]:
    function BindingIdentifier[?Yield, ?Await] ( FormalParameters[~Yield, ~Await] ) { FunctionBody[~Yield, ~Await] }
negative:
  phase: parse
  type: SyntaxError
esid: prod-ModuleItem
flags: [module]
features: [top-level-await]
---*/

$DONOTEVALUATE();

function fn() { await 0; }
