---
layout: page
title: testdriver.js Automation
order: 8.5
---

testdriver.js provides a means to automate tests that cannot be
written purely using web platform APIs.

It is currently supported only for [testharness.js][testharness]
tests.

## API

testdriver.js exposes its API through the `test_driver` variable in
the global scope.

NB: presently, testdriver.js only works in the top-level test browsing
context (and not therefore in any frame or window opened from it).

### `test_driver.click(element)`

This function causes a click to occur on the target element (an
`Element` object), potentially scrolling the document to make it
possible to click it. It returns a `Promise` that resolves after the
click has occured or rejects if the element cannot be clicked (for
example, it is obscured by an element on top of it).

Note that if the element to be clicked does not have a unique ID, the
document must not have any DOM mutations made between the function
being called and the promise settling.


[testharness]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}
