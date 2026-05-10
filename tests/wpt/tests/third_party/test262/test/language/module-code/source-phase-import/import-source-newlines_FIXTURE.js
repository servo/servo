// Copyright (C) 2024 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

// import source in ImportDeclaration may include line terminators
// This test is verified with `import-source.js` in the same directory that
// this file does not raise SyntaxError. Note that a SyntaxError could also
// be raised when the imported module does not have a source phase
// representation (see sec-source-text-module-record-initialize-environment, 7.c.ii).
//
// esid: sec-modules
// info: |
//   ImportDeclaration:
//     import source ImportedBinding FromClause ;
//
//  This test uses all four LineFeed characters in order to completely verify the
//  grammar.
//
//  16.2.1.7.2 GetModuleSource ( )
//  Source Text Module Record provides a GetModuleSource implementation that always returns an abrupt completion indicating that a source phase import is not available.

import "./ensure-linking-error_FIXTURE.js";

import

  source

  y from '<do not resolve>';
