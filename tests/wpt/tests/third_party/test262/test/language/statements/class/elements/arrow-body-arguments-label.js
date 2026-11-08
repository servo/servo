// Copyright (C) 2026 Huáng Jùnliàng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The identifier `arguments` may be used as a label within an arrow function in a class field initializer
info: |
  FieldDefinition : ClassElementName Initializer_opt

  - It is a Syntax Error if Initializer is present and ContainsArguments of
    Initializer is true.

  Static Semantics: ContainsArguments

  IdentifierReference : Identifier

  1. If the StringValue of Identifier is "arguments", return true.
  2. Return false.

  [...]

  LabelIdentifier is not an IdentifierReference, so a label named `arguments`
  does not make ContainsArguments true.
features: [class-fields-public, arrow-function]
---*/

var breakCount = 0;
var continueCount = 0;
var labelledStatement;

class C {
  x = () => {
    arguments: for (;;) {
      breakCount += 1;
      break arguments;
    }

    arguments: for (var i = 0; i < 2; i++) {
      continueCount += 1;
      continue arguments;
    }

    arguments: labelledStatement = 'evaluated';
  };
}

new C().x();

assert.sameValue(breakCount, 1, 'break');
assert.sameValue(continueCount, 2, 'continue');
assert.sameValue(labelledStatement, 'evaluated', 'labelled expression statement');
