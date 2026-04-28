// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations
description: >
    'using' allows BindingIdentifier in lexical bindings
features: [explicit-resource-management]
---*/
{
  using x = null;
}
