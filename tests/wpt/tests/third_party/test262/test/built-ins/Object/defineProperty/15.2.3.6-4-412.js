// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-412
description: >
    ES5 Attributes - [[Value]] field of inherited property of
    [[Prototype]] internal property is correct(Object.create)
---*/

var appointment = {};

Object.defineProperty(appointment, "startTime", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
Object.defineProperty(appointment, "name", {
  value: "NAME",
  writable: true,
  enumerable: true,
  configurable: true
});

var meeting = Object.create(appointment);
Object.defineProperty(meeting, "conferenceCall", {
  value: "In-person meeting",
  writable: true,
  enumerable: true,
  configurable: true
});

var teamMeeting = Object.create(meeting);

var hasOwnProperty = !teamMeeting.hasOwnProperty("name") &&
  !teamMeeting.hasOwnProperty("startTime") &&
  !teamMeeting.hasOwnProperty('conferenceCall');

assert(hasOwnProperty, 'hasOwnProperty !== true');
assert.sameValue(teamMeeting.name, "NAME", 'teamMeeting.name');
assert.sameValue(teamMeeting.startTime, 1001, 'teamMeeting.startTime');
assert.sameValue(teamMeeting.conferenceCall, "In-person meeting", 'teamMeeting.conferenceCall');
