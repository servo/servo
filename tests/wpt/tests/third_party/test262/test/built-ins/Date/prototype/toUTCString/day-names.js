// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-todatestring-day-names
description: Test the names of days
---*/

assert.sameValue("Sun, 23 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-23T00:00:00Z")).toUTCString());
assert.sameValue("Mon, 24 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-24T00:00:00Z")).toUTCString());
assert.sameValue("Tue, 25 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-25T00:00:00Z")).toUTCString());
assert.sameValue("Wed, 26 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-26T00:00:00Z")).toUTCString());
assert.sameValue("Thu, 27 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-27T00:00:00Z")).toUTCString());
assert.sameValue("Fri, 28 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-28T00:00:00Z")).toUTCString());
assert.sameValue("Sat, 29 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-29T00:00:00Z")).toUTCString());
