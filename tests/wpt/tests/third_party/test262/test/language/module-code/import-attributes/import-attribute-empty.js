// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: WithClause in ImportDeclaration may be empty
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
features: [import-attributes, globalThis]
flags: [module]
---*/

import x from './import-attribute-1_FIXTURE.js' with {};
import './import-attribute-2_FIXTURE.js' with {};
export * from './import-attribute-3_FIXTURE.js' with {};

assert.sameValue(x, 262.1);
