// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-modules
es6id: 15.2
description: YieldExpression may not be used directly within ModuleBody
info: |
  Syntax

  Module :
    ModuleBodyopt

  ModuleBody :
    ModuleItemList

  ModuleItemList :
    ModuleItem
    ModuleItemList ModuleItem

  ModuleItem:
    ImportDeclaration
    ExportDeclaration
    StatementListItem[~Yield, ~Return]
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

yield;
