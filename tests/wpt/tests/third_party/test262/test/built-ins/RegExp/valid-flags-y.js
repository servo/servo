// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.3.1
description: The `y` flag is accepted by the RegExp constructor
info: |
    [...]
    10. Return RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize ( obj, pattern, flags )

    [...]
    7. If F contains any code unit other than "g", "i", "m", "u", or "y" or if
       it contains the same code unit more than once, throw a SyntaxError
       exception.
---*/

new RegExp('abc', 'y');
new RegExp('abc', 'gy');
new RegExp('abc', 'iy');
new RegExp('abc', 'my');
new RegExp('abc', 'uy');
new RegExp('abc', 'gimuy');
