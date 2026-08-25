// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  async name token in property and object destructuring pattern
info: bugzilla.mozilla.org/show_bug.cgi?id=1185106
esid: pending
---*/

{
  let a = { async: 10 };
  assert.sameValue(a.async, 10);
}

{
  let a = { async() {} };
  assert.sameValue(a.async instanceof Function, true);
  assert.sameValue(a.async.name, "async");
}

{
  let async = 11;
  let a = { async };
  assert.sameValue(a.async, 11);
}

{
  let { async } = { async: 12 };
  assert.sameValue(async, 12);
}

{
  let { async = 13 } = {};
  assert.sameValue(async, 13);
}

{
  let { async: a = 14 } = {};
  assert.sameValue(a, 14);
}

{
  let { async, other } = { async: 15, other: 16 };
  assert.sameValue(async, 15);
  assert.sameValue(other, 16);

  let a = { async, other };
  assert.sameValue(a.async, 15);
  assert.sameValue(a.other, 16);
}
