// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
description: >
  `false` is a reserved word and cannot be used as an identifier reference.
info: |
  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if StringValue of IdentifierName is the same String
  value as the StringValue of any ReservedWord except for yield.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// It's tempting to write `false = 0`, but that'd be a test to validate `false`
// is not a valid simple assignment target, cf. tests in language/expressions/assignment.
// Also see: sec-semantics-static-semantics-isvalidsimpleassignmenttarget
({false});
