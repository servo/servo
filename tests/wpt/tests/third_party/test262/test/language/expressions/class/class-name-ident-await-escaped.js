// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions
description: >
  `await` with escape sequence is a valid class-name identifier.
info: |
  12.1.1 Static Semantics: Early Errors

  Identifier : IdentifierName but not ReservedWord

  It is a Syntax Error if the goal symbol of the syntactic grammar is Module
  and the StringValue of IdentifierName is "await".
---*/

var C = class aw\u0061it {};
