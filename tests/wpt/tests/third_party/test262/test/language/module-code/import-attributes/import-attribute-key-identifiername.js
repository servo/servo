// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    WithClause in ImportDeclaration may use any valid IdentifierName as a key
esid: sec-modules
info: |
  ImportDeclaration:
    import ModuleSpecifier[no LineTerminator here] WithClause;

  WithClause:
    AttributesKeyword {}
    AttributesKeyword {WithEntries ,opt}

  WithEntries:
    AttributeKey : StringLiteral
    AttributeKey : StringLiteral , WithEntries

  AttributeKey:
    IdentifierName
    StringLiteral
negative:
  phase: resolution
  type: SyntaxError
features: [import-attributes]
flags: [module]
---*/

$DONOTEVALUATE();

import "./ensure-linking-error_FIXTURE.js";

import x from './import-attribute-1_FIXTURE.js' with {if:''};
import './import-attribute-2_FIXTURE.js' with {if:''};
export * from './import-attribute-3_FIXTURE.js' with {if:''};

assert.sameValue(x, 262.1);
