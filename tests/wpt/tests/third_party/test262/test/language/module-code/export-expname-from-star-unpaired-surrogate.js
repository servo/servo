// Copyright (C) 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportSpecifier : ModuleExportName
  esid: prod-ExportSpecifier
info: |
  ModuleExportName : StringLiteral

  It is a Syntax Error if IsStringWellFormedUnicode of the StringValue of
  StringLiteral is *false*.
flags: [module]
features: [arbitrary-module-namespace-names]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

export "*" as "\uD83D" from "./export-expname_FIXTURE.js";
