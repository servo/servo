# Navigation API ordering/transition tests

These are meant to test the ordering between various events and promises, as
well as in some cases how the `navigation.transition` values changes.

Some of them use the `Recorder` framework in `resources/helpers.mjs`, and others
test tricky cases (e.g. reentrancy) in a more ad-hoc way.

<https://github.com/WICG/navigation-api/#complete-event-sequence> is a useful
reference for the intent of these tests.

Note:

* Variants specifically exist for `currententrychange` because an event listener
  existing for `currententrychange` causes code to run, and thus microtasks to run,
  at a very specific point in the navigation-commit lifecycle. We want to test
  that it doesn't impact the ordering.
* Similarly we test that `intercept()` does not change
  the ordering compared to no `intercept()` call, for same-document
  navigations, by trying to ensure most variants have appropriate
  `intercept()` counterparts with similar orderings.

TODOs:

* Also test `popstate` and `hashchange` once
  <https://github.com/whatwg/html/issues/1792> is fixed.
