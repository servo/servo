// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    Set ( [ iterable ] )

    When the Set function is called with optional argument iterable the following steps are taken:

    ...
    5. If iterable is not present, let iterable be undefined.
    6. If iterable is either undefined or null, let iter be undefined.
    ...
    8. If iter is undefined, return set.

---*/


assert.sameValue(new Set().size, 0, "The value of `new Set().size` is `0`");
assert.sameValue(new Set(undefined).size, 0, "The value of `new Set(undefined).size` is `0`");
assert.sameValue(new Set(null).size, 0, "The value of `new Set(null).size` is `0`");
