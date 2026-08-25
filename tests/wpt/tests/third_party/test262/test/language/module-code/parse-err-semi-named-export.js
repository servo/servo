// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    "export NamedExports" declarations require a trailing semicolon or
    LineTerminator
esid: sec-exports
info: |
    ExportDeclaration:
      export * FromClause;
      export * as IdentifierName FromClause;
      export NamedExports FromClause;
      export NamedExports;
      export VariableStatement
      export Declaration
      export default HoistableDeclaration[Default]
      export default ClassDeclaration[Default]
      export default [lookahead ∉ { function, class }] AssignmentExpression[In];
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

export {} null;
