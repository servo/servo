// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-todatestring-month-names
description: Test the names of months
---*/

assert.sameValue("Wed, 01 Jan 2014 00:00:00 GMT",
                 (new Date("2014-01-01T00:00:00Z")).toUTCString());
assert.sameValue("Sat, 01 Feb 2014 00:00:00 GMT",
                 (new Date("2014-02-01T00:00:00Z")).toUTCString());
assert.sameValue("Sat, 01 Mar 2014 00:00:00 GMT",
                 (new Date("2014-03-01T00:00:00Z")).toUTCString());
assert.sameValue("Tue, 01 Apr 2014 00:00:00 GMT",
                 (new Date("2014-04-01T00:00:00Z")).toUTCString());
assert.sameValue("Thu, 01 May 2014 00:00:00 GMT",
                 (new Date("2014-05-01T00:00:00Z")).toUTCString());
assert.sameValue("Sun, 01 Jun 2014 00:00:00 GMT",
                 (new Date("2014-06-01T00:00:00Z")).toUTCString());
assert.sameValue("Tue, 01 Jul 2014 00:00:00 GMT",
                 (new Date("2014-07-01T00:00:00Z")).toUTCString());
assert.sameValue("Fri, 01 Aug 2014 00:00:00 GMT",
                 (new Date("2014-08-01T00:00:00Z")).toUTCString());
assert.sameValue("Mon, 01 Sep 2014 00:00:00 GMT",
                 (new Date("2014-09-01T00:00:00Z")).toUTCString());
assert.sameValue("Wed, 01 Oct 2014 00:00:00 GMT",
                 (new Date("2014-10-01T00:00:00Z")).toUTCString());
assert.sameValue("Sat, 01 Nov 2014 00:00:00 GMT",
                 (new Date("2014-11-01T00:00:00Z")).toUTCString());
assert.sameValue("Mon, 01 Dec 2014 00:00:00 GMT",
                 (new Date("2014-12-01T00:00:00Z")).toUTCString());
