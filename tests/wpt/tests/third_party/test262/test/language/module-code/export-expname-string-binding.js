// Copyright (C) 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ExportDeclaration : `export` NamedExports `;`
  esid: sec-exports-static-semantics-early-errors
info: |
  ExportDeclaration : `export` NamedExports `;`

  It is a Syntax Error if ReferencedBindings of |NamedExports| contains any
  |ModuleExportName|.
flags: [module]
features: [arbitrary-module-namespace-names]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

export { "foo" as "bar" }

function foo() {}
