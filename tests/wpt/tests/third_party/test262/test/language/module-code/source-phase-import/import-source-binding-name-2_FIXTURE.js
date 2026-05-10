// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// ImportBinding in ImportDeclaration may be 'source' and 'from'
// This test is verified with `import-source.js` in the same directory that
// this file does not raise SyntaxError. Note that a SyntaxError could also
// be raised when the imported module does not have a source phase
// representation (see sec-source-text-module-record-initialize-environment, 7.c.ii).
//
// esid: sec-modules
// info: |
//   ImportDeclaration:
//     import source ImportedBinding FromClause ;

import "./ensure-linking-error_FIXTURE.js";

import source source from '<do not resolve>';
import source from from '<do not resolve>';
