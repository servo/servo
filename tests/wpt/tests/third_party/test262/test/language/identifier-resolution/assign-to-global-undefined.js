// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializeboundname
description: >
  In strict mode code, attempts to assign to an unresolvable reference must throw a ReferenceError exception
info: |
  via sec-putvalue

  If IsUnresolvableReference(V) is true, then
    If IsStrictReference(V) is true, then
      Throw a ReferenceError exception.

flags: [onlyStrict]
negative:
  phase: runtime
  type: ReferenceError
---*/

undeclared = (this.undeclared = 5);
