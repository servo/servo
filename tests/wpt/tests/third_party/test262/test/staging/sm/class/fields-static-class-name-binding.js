// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Static field initialisers can access the inner name binding for class definitions.
{
  class C {
    static field = C;
  }
  assert.sameValue(C.field, C);
}
{
  let C = class Inner {
    static field = Inner;
  };
  assert.sameValue(C.field, C);
}

// Static field initialisers can't access the outer name binding for class expressions
// before it has been initialised.
{
  assert.throws(ReferenceError, () => {
    let C = class {
      static field = C;
    };
  });
}

// Static field initialisers can access the outer name binding for class expressions after
// the binding has been initialised
{
  let C = class {
    static field = () => C;
  };
  assert.sameValue(C.field(), C);
}

// Static field initialiser expressions always resolve the inner name binding.
{
  class C {
    static field = () => C;
  }
  assert.sameValue(C.field(), C);

  const D = C;
  C = null;

  assert.sameValue(D.field(), D);
}
{
  let C = class Inner {
    static field = () => Inner;
  }
  assert.sameValue(C.field(), C);

  const D = C;
  C = null;

  assert.sameValue(D.field(), D);
}

