// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: >
    It is a Syntax Error if we declare a static private setter and a private instance getter
features: [class-static-methods-private, class-methods-private]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class C {
  static set #f(v) {}
  get #f() {}
}

