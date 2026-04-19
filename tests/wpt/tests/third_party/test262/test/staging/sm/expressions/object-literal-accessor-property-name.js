/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Permit numbers and strings containing numbers as accessor property names
info: bugzilla.mozilla.org/show_bug.cgi?id=715682
esid: pending
---*/

({ get "0"() { } });
({ get 0() { } });
({ get 0.0() { } });
({ get 0.() { } });
({ get 1.() { } });
({ get 5.2322341234123() { } });

({ set "0"(q) { } });
({ set 0(q) { } });
({ set 0.0(q) { } });
({ set 0.(q) { } });
({ set 1.(q) { } });
({ set 5.2322341234123(q) { } });
