# User activation tests

Most files here test the user activation states themselves: which input events
grant them (`activation-trigger-*.html`), how they propagate between frames
(`propagation-*.html`), how they are reset by a navigation
(`navigation-state-reset-*.html`), and the `navigator.userActivation`
interface.

The rest test sticky activation **carry-over**: a navigation, reload or
traversal may hand the sticky activation of the document being replaced to the
one replacing it. See
[whatwg/html#11454](https://github.com/whatwg/html/pull/11454)

## The sticky activation carry-over conditions

The carry-over happens when all of the following are true:

1. `displayedDocument`'s relevant global object has sticky activation;
2. `displayedDocument`'s origin is same origin with `targetEntry`'s document's
   origin;
3. `targetEntry`'s document's `was created via cross-origin redirects` is
   false; and
4. `navigationType` is `"reload"`, or both of:
   - 4a. `targetEntry`'s document state's `initiator origin` is not null and is
     same origin with `targetEntry`'s document's origin; and
   - 4b. `targetEntry`'s document state's `was created via cross-origin
     redirects` is false.

Only the sticky bit is ever carried over, and it is only ever set, never
cleared. Conditions 3 and 4b read fields of the same name on two different
objects: 3 dies with the `Document`, while 4b latches on the entry's document
state and is never cleared.


**No carry-over condition blocking it.**

- `navigate-to-sameorigin.html`: a link to another document on the same origin.
- `navigate-to-sameorigin-redirect.html`: the same through a redirect that stays
  on that origin, since only redirects that cross origins are recorded.
- `replace-sameorigin.html`: `location.replace()`, which the conditions do not
  tell apart from a push.
- `reload-sameorigin.html`: a reload.
- `traverse-to-sameorigin.https.html`: a traversal back to a same-origin
  document.
- `traverse-to-pushed-entry.https.html`: a traversal past an entry added by
  `pushState`, so that the entry it lands on is not the one its document was
  last shown for.
- `traverse-same-document-keeps-activation.html`: a traversal between two
  entries of one document, where no carry-over runs at all and the document has
  to keep the activation it earned.
- `iframe-clicked-after-navigation-started.html`: an iframe clicked while the
  navigation replacing its document is still in flight, held back by `trickle`.
  The conditions are read when the history step is applied, so activation
  gained by then counts; the destination's transient bit says which document
  the click landed in.
- `navigate-sameorigin-chain.html`: two navigations and a reload after a single
  click, where the document driving each step after the first only has sticky
  activation because it was carried over.
- `navigate-to-javascript-url.html`: a `javascript:` URL, whose document the
  spec gives the initiator's origin and a document state that records no
  redirect, so none of the conditions rule it out. The document reports its own
  state, since it replaces the executor that would otherwise answer for the
  window.
- `navigate-with-coop-group-switch.https.html`: a navigation between two
  same-origin documents whose `Cross-Origin-Opener-Policy` values do not match,
  which switches the browsing context group. None of the conditions reads a
  browsing context, and the entry the carry-over is decided from belongs to the
  navigable, which the switch does not replace. The window loses its opener, so
  the state is read through the executor rather than through a window
  reference.

**Carry-over condition 2, the two documents being same origin.**
- `navigate-to-crossorigin.html`: a cross-origin window navigates the popup to a
  URL on that window's own origin, which leaves the entry's initiator same
  origin with the destination so that only condition 2 is left.
- `reload-to-crossorigin.html`: the entry is pointed at a URL that redirects to
  another origin and reloaded, which also shows that the exemption for reloads
  does not cover this condition.
- `traverse-to-crossorigin.html`: a traversal back to a cross-origin document.

**Carry-over condition 3, the new document having followed a cross-origin
redirect.**
- `navigate-to-crossorigin-redirect.html`: a link that goes cross origin and
  redirects back, so the destination is same origin with the document being
  replaced and only the redirect is left.
- `reload-through-crossorigin-redirect.html`: the same for a reload, which is
  not exempt from this condition either.
- `traverse-after-crossorigin-redirect.https.html`: a traversal back to a
  document opened through a cross-origin redirect. The `?store` variant is the
  one this condition decides, because the document comes back from the
  back/forward cache still carrying its own bit.
- `traverse-refetch-through-crossorigin-redirect.https.html`: the other way
  round, where the entry was reached without a redirect and only the re-fetch
  the traversal has to make follows one, so what blocks the carry-over is the
  document the traversal creates rather than anything the entry remembered.

**Carry-over condition 4a, the entry's initiator.**

- `navigate-initiated-by-crossorigin.html`: a cross-origin window navigates the
  popup to a same-origin URL.
- `navigate-top-initiated-by-crossorigin-iframe.html`: a cross-origin iframe
  sets `top.location`.
- `iframe-navigated-by-crossorigin-parent.html`: a parent navigates a
  cross-origin iframe by setting `src`.
- `traverse-to-entry-initiated-by-crossorigin.https.html`: a traversal back to
  an entry a cross-origin window navigated to, read long after that window is
  gone.
- `iframe-traverse-to-entry-initiated-by-crossorigin-parent.html`: a traversal
  back to an iframe's first entry, which its parent created.
- `traverse-forward-to-entry-initiated-by-crossorigin.html`: a traversal
  forward to an entry a cross-origin document navigated to, leaving a document
  that is same origin and freshly clicked, so that the entry's record is all
  that is left to rule it out.
- `iframe-navigated-by-itself-in-crossorigin-parent.html`: satisfied, because
  the iframe navigates itself inside a cross-origin parent.
- `traverse-started-by-crossorigin-iframe.html`: satisfied, because a
  cross-origin iframe is merely the one that calls `history.back()` — the
  condition reads what the entry recorded rather than who asked.

**Carry-over condition 4b, and the reload exemption.**

- `traverse-after-crossorigin-redirect.https.html?no-store`: the document is
  populated afresh by a fetch that follows no redirect, so its own bit is clean
  and the entry's latched bit is all that is left.
- `reload-of-crossorigin-redirected-entry.html`: a reload of an entry opened
  through a cross-origin redirect, carried over, over 4b.
- `reload-of-crossorigin-initiated-entry.html`: through `location.reload()` and
  through `history.go(0)`, which the spec makes a reload of the navigable rather
  than a traversal, so it takes the same exemption. A reload of an entry a
  cross-origin window navigated to, carried over, over 4a.

**Carry-over condition 1, the predecessor having sticky activation.**

- `iframe-recreated-by-traversal.https.html`: a traversal back to a window
  holding an iframe. The window gets the carry-over either way, but the iframe
  only keeps what it earned in the `?store` variant, where it comes back with
  the page; in `?no-store` it is recreated, and a frame that did not exist while
  the other document was shown has no predecessor `Window` to copy from.
  whatwg/html#11454 calls this out as the one case the change does not cover.

**Nothing carried over, and nothing lost.**

- `traverse-keeps-own-activation.https.html`: a traversal back from a
  cross-origin document to one that was clicked. Condition 2 blocks the
  carry-over, so the document the traversal returns to has to keep the sticky
  activation it earned, which `?store` reads off the restored document.
  `?no-store` recreates it instead, and it starts out with nothing: the bit is
  not something the entry stores on its document's behalf.
- `traverse-aborted-by-204.https.html`: a traversal to an entry whose URL
  answers 204, which leaves that entry's document null, so the window never
  goes cross-document and keeps the activation it earned where it is. An
  aborted traversal reports nothing to wait on, which is why this one waits.

**When the bit becomes observable.** Read at the earliest points the spec
allows.

- `traverse-carry-over-before-pageshow.https.html`: an inline script in the
  document's head and a `pageshow` handler, in a document that is given no
  activation of its own so that whatever they see came from the carry-over. `?no-store` reads the recreated document's first
  script, which the spec sets the bit before, and `?store` reads the
  reactivation's persisted `pageshow`.

**Origins a document does not have of its own.**

- `iframe-sandboxed-navigated-by-itself.html`: a sandboxed iframe, clicked and
  then navigating itself, gets no carry-over. `determine the origin` hands out a
  new opaque origin whenever the sandboxed origin browsing context flag is set,
  before it reaches the cases that inherit one, and two different opaque origins
  are never same origin. So the same-origin condition and the entry's initiator
  fail together, and no arrangement of a sandboxed document can satisfy either:
  what the test pins is the guarantee rather than one condition. It drives the
  iframe with `/common/dispatcher/executor.html` and `send()`/`receive()`, as
  the COOP sandbox tests do, rather than with `RemoteContext.execute_script()`
  like the tests above.
- `iframe-navigated-to-inherited-origin.html`: an iframe navigated to
  `about:blank`, to a `srcdoc` document or to a `blob:` URL, each of which takes
  this page's origin, which leaves the destination same origin with both the
  document it replaces and the initiator. The carry-over happens in all three.
- `navigate-to-csp-sandboxed-document.html`: a window navigated to a document
  whose response carries `Content-Security-Policy: sandbox`, which is how a
  top-level document gets the opaque origin the `sandbox` attribute gives a
  frame. Only the destination is sandboxed, so that the window is still
  reachable through the handle while it is clicked.

**Other entry points.**

- `navigation-api-entry-points.html`: the same navigations asked for through
  `navigation.navigate()`, `navigation.reload()`, `navigation.back()`,
  `navigation.traverseTo()` and `history.go(0)`. All of them are specified to
  reach "apply the history step", so the expectations come from the tests above
  and what each variant checks is that its entry point gets there.


## Not covered yet

- The reactivation path, which the `?store` variants reach by traversing to a
  document the back/forward cache kept, needs a cache that can get there, and
  those variants report `PRECONDITION_FAILED` where it cannot. A window opened
  without `noopener` is often not eligible, and
  `traverse-to-pushed-entry.https.html?store` additionally needs a cache that
  can restore a document into a different entry of its own.
- Cases the spec has not settled: prerendering, which this standard only names
  as a speculation rules key and leaves to implementations, and initial
  `about:blank` `Window` reuse, which
  [whatwg/html#13015](https://github.com/whatwg/html/issues/13015) tracks.
