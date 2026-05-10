// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  WithClause may not have duplicate keys (import declaration with binding)
esid: sec-modules
info: |
  WithClause: AttributesKeyword { WithEntries,opt }

  - It is a Syntax Error if WithClauseToAttributes of WithClause has two
    entries a and b such that a.[[Key]] is b.[[Key]].
features: [import-attributes]
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

import x from './import-attribute-1_FIXTURE.js' with {
  type: 'json',
  'typ\u0065': ''
};
