// Copyright 2011 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2_A8_T1
description: check if "caller" poisoning poisons  "in" too
flags: [onlyStrict]
---*/

'caller' in function() {};
