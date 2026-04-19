// Copyright (C) 2019 Adrian Heine. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: In non-strict mode, duplicate LexicallyDeclaredNames in a block are allowed if they are bound by FunctionDeclarations
esid: sec-block-duplicates-allowed-static-semantics
es6id: B.3.3.4
flags: [noStrict]
info: |
    B.3.3.4 Changes to Block Static Semantics: Early Errors

    For web browser compatibility, that rule is modified with the addition of the **highlighted** text:

    Block: {StatementList}

    It is a Syntax Error if the LexicallyDeclaredNames of StatementList contains any duplicate entries, **unless the source code matching this production is not strict mode code and the duplicate entries are only bound by FunctionDeclarations**.
---*/

{ function a() {} function a() {} }
