/*---
description: Verify async test failure handling
flags: [async]
---*/

$DONE(new Error("Explicit async failure"));
