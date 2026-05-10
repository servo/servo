// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Instance field initialisers can access the inner name binding for class definitions.
{
  class C {
    field = C;
  }
  assert.sameValue(new C().field, C);
}
{
  let C = class Inner {
    field = Inner;
  };
  assert.sameValue(new C().field, C);
}

// Instance field initialiser expressions always resolve the inner name binding.
{
  class C {
    field = () => C;
  }
  assert.sameValue(new C().field(), C);

  const D = C;
  C = null;

  assert.sameValue(new D().field(), D);
}
{
  let C = class Inner {
    field = () => Inner;
  }
  assert.sameValue(new C().field(), C);

  const D = C;
  C = null;

  assert.sameValue(new D().field(), D);
}

