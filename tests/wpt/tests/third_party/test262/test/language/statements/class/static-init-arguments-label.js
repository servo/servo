// Copyright (C) 2026 Huáng Jùnliàng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The identifier `arguments` may be used as a label within a class static block
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsArguments of ClassStaticBlockStatementList
    is true.

  Static Semantics: ContainsArguments

  IdentifierReference : Identifier

  1. If the StringValue of Identifier is "arguments", return true.
  2. Return false.

  [...]

  LabelIdentifier is not an IdentifierReference, so a label named `arguments`
  does not make ContainsArguments true.
features: [class-static-block]
---*/

var breakCount = 0;
var continueCount = 0;
var labelledStatement;

class C {
  static {
    arguments: for (;;) {
      breakCount += 1;
      break arguments;
    }

    arguments: for (var i = 0; i < 2; i++) {
      continueCount += 1;
      continue arguments;
    }

    arguments: labelledStatement = 'evaluated';
  }
}

assert.sameValue(breakCount, 1, 'break');
assert.sameValue(continueCount, 2, 'continue');
assert.sameValue(labelledStatement, 'evaluated', 'labelled expression statement');
