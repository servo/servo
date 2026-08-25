// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: ASI test in field declarations -- error when generator interpreted as multiplication
esid: sec-automatic-semicolon-insertion
features: [class, class-fields-public, generators]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var C = class {
  x = 42
  *gen() {}
}
