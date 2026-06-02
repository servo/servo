# How to write back-forward cache tests

In the back-forward cache tests, the main test HTML usually:

1. Opens new executor Windows using `window.open()` + `noopener` option,
   because less isolated Windows (e.g. iframes and `window.open()` without
   `noopener` option) are often not eligible for back-forward cache (e.g.
   in Chromium).
2. Injects scripts to the executor Windows and receives the results via
   `RemoteContext.execute_script()` by
   [/common/dispatcher](../../../../common/dispatcher/README.md).
   Follow the semantics and guideline described there.

Back-forward cache specific helpers are in:

- [resources/executor.html](resources/executor.html):
  The BFCache-specific executor and contains helpers for executors.
- [resources/helper.sub.js](resources/helper.sub.js):
  Helpers for main test HTMLs.

We must ensure that injected scripts are evaluated only after page load
(more precisely, the first `pageshow` event) and not during navigation,
to prevent unexpected interference between injected scripts, in-flight fetch
requests behind `RemoteContext.execute_script()`, navigation and back-forward
cache. To ensure this,

- Call `await remoteContext.execute_script(waitForPageShow)` before any
  other scripts are injected to the remote context, and
- Call `prepareNavigation(callback)` synchronously from the script injected
  by `RemoteContext.execute_script()`, and trigger navigation on or after the
  callback is called.

In typical A-B-A scenarios (where we navigate from Page A to Page B and then
navigate back to Page A, assuming Page A is (or isn't) in BFCache),

- Call `prepareNavigation()` on the executor, and then navigate to B, and then
  navigate back to Page A.
- Call `assert_bfcached()` or `assert_not_bfcached()` on the main test HTML, to
  check the BFCache status. This is important to do to ensure the test would
  not fail normally and instead result in `PRECONDITION_FAILED` if the page is
  unexpectedly bfcached/not bfcached.
- Check other test expectations on the main test HTML,

as in [events.html](./events.html) and `runEventTest()` in
[resources/helper.sub.js](resources/helper.sub.js).

# Asserting PRECONDITION_FAILED for unexpected BFCache eligibility

Browsers are not actually obliged to put pages in BFCache after navigations, so
BFCache WPTs shouldn't result in `FAILED` if it expects a certain case to be
supported by BFCache. But, it is still useful to test those cases in the
browsers that do support BFCache for that case.

To distinguish genuine failures from just not using BFCache, we use
`assert_bfcached()` and `assert_not_bfcached()` which result in
`PRECONDITION_FAILED` rather than `FAILED`. that should be put in the
expectations for the failing tests (instead of marking it as `FAILED` or
skipping the test). This means if the test starts passing (e.g. if we start
BFCaching in the case being tested), we will notice that the output changed from
`PRECONDITION_FAILED` to `PASS`.
