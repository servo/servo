// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-identifiers-static-semantics-early-errors
description: >
  If the source code matching this production is strict code, it is a
  Syntax Error if BindingIdentifier is the IdentifierName eval.
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
features: [async-iteration]
---*/

$DONOTEVALUATE();

(async function* eval() { });
