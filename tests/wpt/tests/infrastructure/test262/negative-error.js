/*---
description: A negative test that throws a different error than expected.
negative:
  phase: runtime
  type: TypeError
---*/

throw new RangeError();
