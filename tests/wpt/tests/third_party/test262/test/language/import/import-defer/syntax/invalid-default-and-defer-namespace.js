// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-imports
description: >
  `import defer` cannot be used if there are both a default and namespace bindings
info: |
  ImportDeclaration :
    `import` ImportClause FromClause `;`
    `import` `defer` NameSpaceImport FromClause `;`
    `import` ModuleSpecifier `;`

  ImportClause :
    ImportedDefaultBinding
    NameSpaceImport
    NamedImports
    ImportedDefaultBinding `,` NameSpaceImport
    ImportedDefaultBinding `,` NamedImports

  NameSpaceImport :
    `*` `as` ImportedBinding

features: [import-defer]

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

import defer x, * as ns from "./dep_FIXTURE.js";
