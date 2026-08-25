// Copyright (C) 2017 Lucas Azzola. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Lucas Azzola
esid: pending
description: try/catch/finally syntax with omission of the catch binding
features: [optional-catch-binding]
info: |
  Optional Catch Binding

  Catch[Yield, Await, Return]:
    (...)
    catch Block[?Yield, ?Await, ?Return]
---*/

try {} catch {} finally {}
