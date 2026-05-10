// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ImportedBinding is a BindingIdentifier and cannot be "eval"
esid: sec-imports
info: |
    ImportSpecifier :
      ImportedBinding
      IdentifierName as ImportedBinding

    ImportedBinding :
      BindingIdentifier

    12.1.1 Static Semantics : Early Errors

    BindingIdentifier : Identifier

    - It is a Syntax Error if the code matched by this production is contained
      in strict mode code and the StringValue of Identifier is "arguments" or
      "eval".
negative:
  phase: parse
  type: SyntaxError
flags: [module]
---*/

$DONOTEVALUATE();

// Create an appropriately-named ExportEntry in order to avoid false positives
// (e.g. cases where the implementation does not generate the expected early
// error but does produce a SyntaxError for unresolvable bindings).
var x;
export { x as eval };

import { eval } from './early-import-eval.js';
