// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-exports
description: >
  `export defer` is not valid
info: |
  ExportDeclaration :
    `export` ExportFromClause FromClause `;`
    `export` NamedExports `;`
    `export` VariableStatement
    `export` Declaration
    `export` `default` HoistableDeclaration
    `export` `default` ClassDeclaration
    `export` `default` [lookahead ∉ { function, async [no LineTerminator here] function, class }] AssignmentExpression `;`

    ExportFromClause :
      `*`
      `*` `as` ModuleExportName
      NamedExports

features: [import-defer]

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

export defer * as ns from "./dep_FIXTURE.js";
