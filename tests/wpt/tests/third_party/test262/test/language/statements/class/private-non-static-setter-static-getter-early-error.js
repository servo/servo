// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: >
    It is a Syntax Error if we declare a private instance setter and a static private getter
features: [class-static-methods-private, class-methods-private]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class C {
  set #f(v) {}
  static get #f() {}
}

