// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Named groups can be forward references.
esid: sec-atomescape
features: [regexp-named-groups]
---*/

assert(/\k<a>(?<a>x)/.test("x"));
