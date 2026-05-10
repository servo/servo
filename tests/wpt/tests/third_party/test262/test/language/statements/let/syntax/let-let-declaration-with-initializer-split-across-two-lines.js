// Copyright (C) 2015 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Jeff Walden
es6id: 13.3.1.1
description: >
    let: |let let| split across two lines is not subject to automatic semicolon insertion.
info: |
  |let| followed by a name is a lexical declaration.  This is so even if the
  name is on a new line.  ASI applies *only* if an offending token not allowed
  by the grammar is encountered, and there's no [no LineTerminator here]
  restriction in LexicalDeclaration or ForDeclaration forbidding a line break.

  It's a tricky point, but this is true *even if* the name is "let", a name that
  can't be bound by LexicalDeclaration or ForDeclaration.  Per 5.3, static
  semantics early errors are validated *after* determining productions matching
  the source text.

  So in this testcase, the eval text matches LexicalDeclaration.  No ASI occurs,
  because "let\nlet = ..." matches LexicalDeclaration before static semantics
  are considered.  *Then* 13.3.1.1's static semantics for the LexicalDeclaration
  just chosen, per 5.3, are validated to recognize the Script as invalid.  Thus
  the eval script can't be evaluated, and a SyntaxError is thrown.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

let  // start of a LexicalDeclaration, *not* an ASI opportunity
let = "irrelevant initializer";
