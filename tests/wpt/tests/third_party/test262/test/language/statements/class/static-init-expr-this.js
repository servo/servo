// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-operations-on-objects
description: The "this" value within a static initialization block is the class
info: |
  2.1.1 EvaluateStaticBlock ( receiver , blockRecord )

    1. Assert: Type(receiver) is Object.
    2. Assert: blockRecord is a ClassStaticBlockDefinition Record.
    3. Perform ? Call(blockRecord.[[Body]], receiver).
features: [class-static-block]
---*/

var value;

class C {
  static {
    value = this;
  }
}

assert.sameValue(value, C);
