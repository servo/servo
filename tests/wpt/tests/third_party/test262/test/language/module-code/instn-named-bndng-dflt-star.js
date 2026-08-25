// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    An ImportClause may contain both an ImportedDefaultBinding and a
    NameSpaceImport
esid: sec-imports
info: |
    Syntax

    ImportClause:
      ImportedDefaultBinding
      NameSpaceImport
      NamedImports
      ImportedDefaultBinding , NameSpaceImport
      ImportedDefaultBinding , NamedImports
flags: [module]
---*/

assert.throws(ReferenceError, function() {
  typeof x;
});

assert('attr' in ns);

export default 3;
export var attr;
import x, * as ns from './instn-named-bndng-dflt-star.js';
