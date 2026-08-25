// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Jeff Walden <jwalden+code@mit.edu>
esid: sec-let-and-const-declarations
description: >
  `let await` does not permit ASI in between, as `await` is a BindingIdentifier
info: |
  `await` is a perfectly cromulent binding name in any context grammatically, just
  prohibited by static semantics in some contexts.  Therefore ASI can never apply
  between `let` (where a LexicalDeclaration is permitted) and `await`,
  so a subsequent `0` where `=` was expected is a syntax error.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

async function f() {
    let
    await 0;
}
