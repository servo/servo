// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportDeclaration : HoistableDeclaration : FunctionDeclaration
  esid: prod-HoistableDeclaration
info: |
  ExportDeclaration :
    HoistableDeclaration[Yield, Await, Default]:

  HoistableDeclaration[Yield, Await, Default]:
    FunctionDeclaration[?Yield, ?Await, ?Default]

flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class F {}
export default function F() {}

