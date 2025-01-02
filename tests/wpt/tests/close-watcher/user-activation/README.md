# Close watcher user activation tests

These tests are all in separate files (or test variants) because we need to be
sure we're starting from zero user activation.

## Note on variants vs. `-dialog` and `-CloseWatcher` files

We endeavor to have all the tests in these files cover both `<dialog>` elements
and the `CloseWatcher` API. (And sometimes the `popover=""` attribute.)

When the test expectations are the same for both `<dialog>` and `CloseWatcher`,
we use WPT's variants feature.

However, in some cases different expectations are necessary. This is because
`<dialog>`s queue a task to fire their `close` event, and do not queue a task
to fire their `cancel` event. Thus, when you have two `<dialog>`s grouped
together, you get the somewhat-strange behavior of both `cancel`s firing first,
then both `close`s. Whereas `CloseWatcher`s do not have this issue; both events
fire synchronously.

(Note that scheduling the `cancel` event for `<dialog>`s is not really possible,
since it would then fire after the dialog has been closed in the DOM and
visually. So the only reasonable fix for this would be to stop scheduling the
`close` event for dialogs. That's risky from a compat standpoint, so for now,
we test the strange behavior.)
