// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  `with` AttributesKeyword in WithClause in ImportDeclaration can
  be preceded by a line terminator
esid: sec-modules
info: |
  ImportDeclaration:
    import ModuleSpecifier WithClause;

  WithClause:
    AttributesKeyword {}
    AttributesKeyword { WithEntries ,opt }

  AttributesKeyword:
    with
    [no LineTerminator here] assert

negative:
  phase: resolution
  type: SyntaxError
features: [import-attributes, globalThis]
flags: [module, raw]
---*/

throw "Test262: This statement should not be evaluated.";

import "./ensure-linking-error_FIXTURE.js";

import * as x from './import-attribute-1_FIXTURE.js'
with
{};
