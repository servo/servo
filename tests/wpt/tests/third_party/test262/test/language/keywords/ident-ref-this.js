// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-identifiers-static-semantics-early-errors
es5id: 7.6.1.1_A1.18
description: Checking if execution of "this=1" fails
info: |
  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if StringValue of IdentifierName is the same String
  value as the StringValue of any ReservedWord except for yield.
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// It's tempting to write `this = 0`, but that'd be a test to validate `this`
// is not a valid simple assignment target, cf. tests in language/expressions/assignment.
// Also see: sec-semantics-static-semantics-isvalidsimpleassignmenttarget
({this});
