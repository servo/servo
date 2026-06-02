# Making a Testing Plan

When contributing to a project as large and open-ended as WPT, it's easy to get
lost in the details. It can be helpful to start by making a rough list of tests
you intend to write. That plan will let you anticipate how much work will be
involved, and it will help you stay focused once you begin.

Many people come to WPT with a general testing goal in mind:

- specification authors often want to test for new spec text
- browser maintainers often want to test new features or fixes to existing
  features
- web developers often want to test discrepancies between browsers on their web
  applications

(If you don't have any particular goal, we can help you get started. Check out
[the issues labeled with `type:missing-coverage` on
GitHub.com](https://github.com/web-platform-tests/wpt/labels/type%3Amissing-coverage).
Leave a comment if you'd like to get started with one, and don't hesitate to
ask clarifying questions!)

This guide will help you write a testing plan by:

1. showing you how to use the specifications to learn what kinds of tests will
   be most helpful
2. developing your sense for what *doesn't* need to be tested
3. demonstrating methods for figuring out which tests (if any) have already
   been written for WPT

The level of detail in useful testing plans can vary widely. From [a list of
specific
cases](https://github.com/web-platform-tests/wpt/issues/6980#issue-252255894),
to [an outline of important coverage
areas](https://github.com/web-platform-tests/wpt/issues/18549#issuecomment-522631537),
to [an annotated version of the specification under
test](https://rwaldron.github.io/webrtc-pc/), the appropriate fidelity depends
on your needs, so you can be as precise as you feel is helpful.

## Understanding the "testing surface"

Web platform specifications are instructions about how a feature should work.
They're critical for implementers to "build the right thing," but they are also
important for anyone writing tests. We can use the same instructions to infer
what kinds of tests would be likely to detect mistakes. Here are a few common
patterns in specification text and the kind of tests they suggest.

### Input sources

Algorithms may accept input from many sources. Modifying the input is the most
direct way we can influence the browser's behavior and verify that it matches
the specifications. That's why it's helpful to be able to recognize different
sources of input.

```eval_rst
================ ==============================================================
Type of feature  Potential input sources
================ ==============================================================
JavaScript       parameters, `context object <https://dom.spec.whatwg.org/#context-object>`_
HTML             element content, attributes, attribute values
CSS              selector strings, property values, markup
================ ==============================================================
```

Determine which input sources are relevant for your chosen feature, and build a
list of values which seem worthwhile to test (keep reading for advice on
identifying worthwhile values). For features that accept multiple sources of
input, remember that the interaction between values can often produce
interesting results. Every value you identify should go into your testing plan.

*Example:* This is the first step of the `Notification` constructor from [the
Notifications standard](https://notifications.spec.whatwg.org/#constructors):

> The Notification(title, options) constructor, when invoked, must run these steps:
>
> 1. If the [current global
>    object](https://html.spec.whatwg.org/multipage/webappapis.html#current-global-object)
>    is a
>    [ServiceWorkerGlobalScope](https://w3c.github.io/ServiceWorker/#serviceworkerglobalscope)
>    object, then [throw](https://webidl.spec.whatwg.org/#dfn-throw) a
>    `TypeError` exception.
> 2. Let *notification* be the result of [creating a
>    notification](https://notifications.spec.whatwg.org/#create-a-notification)
>    given *title* and *options*. Rethrow any exceptions.
>
> [...]

A thorough test suite for this constructor will include tests for the behavior
of many different values of the *title* parameter and the *options* parameter.
Choosing those values can be a challenge unto itself--see [Avoid Excessive
Breadth](#avoid-excessive-breadth) for advice.

### Browser state

The state of the browser may also influence algorithm behavior. Examples
include the current document, the dimensions of the viewport, and the entries
in the browsing history. Just like with direct input, a thorough set of tests
will likely need to control these values. Browser state is often more expensive
to manipulate (whether in terms of code, execution time, or system resources),
and you may want to design your tests to mitigate these costs (e.g. by writing
many subtests from the same state).

You may not be able to control all relevant aspects of the browser's state.
[The `type:untestable`
label](https://github.com/web-platform-tests/wpt/issues?q=is%3Aopen+is%3Aissue+label%3Atype%3Auntestable)
includes issues for web platform features which cannot be controlled in a
cross-browser way. You should include tests like these in your plan both to
communicate your intention and to remind you when/if testing solutions become
available.

*Example:* In [the `Notification` constructor referenced
above](https://notifications.spec.whatwg.org/#constructors), the type of "the
current global object" is also a form of input. The test suite should include
tests which execute with different types of global objects.

### Branches

When an algorithm branches based on some condition, that's an indication of an
interesting behavior that might be missed. Your testing plan should have at
least one test that verifies the behavior when the branch is taken and at least
one more test that verifies the behavior when the branch is *not* taken.

*Example:* The following algorithm from [the HTML
standard](https://html.spec.whatwg.org/) describes how the
`localStorage.getItem` method works:

> The `getItem`(*key*) method must return the current value associated with the
> given *key*. If the given *key* does not exist in the list associated with
> the object then this method must return null.

This algorithm exhibits different behavior depending on whether or not an item
exists at the provided key. To test this thoroughly, we would write two tests:
one test would verify that `null` is returned when there is no item at the
provided key, and the other test would verify that an item we previously stored
was correctly retrieved when we called the method with its name.

### Sequence

Even without branching, the interplay between sequential algorithm steps can
suggest interesting test cases. If two steps have observable side-effects, then
it can be useful to verify they happen in the correct order.

Most of the time, step sequence is implicit in the nature of the
algorithm--each step operates on the result of the step that precedes it, so
verifying the end result implicitly verifies the sequence of the steps. But
sometimes, the order of two steps isn't particularly relevant to the result of
the overall algorithm. This makes it easier for implementations to diverge.

There are many common patterns where step sequence is observable but not
necessarily inherent to the correctness of the algorithm:

- input validation (when an algorithm verifies that two or more input values
  satisfy some criteria)
- event dispatch (when an algorithm
  [fires](https://dom.spec.whatwg.org/#concept-event-fire) two or more events)
- object property access (when an algorithm retrieves two or more property
  values from an object provided as input)

*Example:* The following text is an abbreviated excerpt of the algorithm that
runs during drag operations (from [the HTML
specification](https://html.spec.whatwg.org/multipage/dnd.html#dnd)):

> [...]
> 4. Otherwise, if the user ended the drag-and-drop operation (e.g. by
>    releasing the mouse button in a mouse-driven drag-and-drop interface), or
>    if the `drag` event was canceled, then this will be the last iteration.
>    Run the following steps, then stop the drag-and-drop operation:
>    1. If the [current drag
>       operation](https://html.spec.whatwg.org/multipage/dnd.html#current-drag-operation)
>       is "`none`" (no drag operation) [...] Otherwise, the drag operation
>       might be a success; run these substeps:
>       1. Let *dropped* be true.
>       2. If the [current target
>          element](https://html.spec.whatwg.org/multipage/dnd.html#current-target-element)
>          is a DOM element, [fire a DND
>          event](https://html.spec.whatwg.org/multipage/dnd.html#fire-a-dnd-event)
>          named `drop` at it; otherwise, use platform-specific conventions for
>          indicating a drop.
>       3. [...]
>    2. [Fire a DND
>       event](https://html.spec.whatwg.org/multipage/dnd.html#fire-a-dnd-event)
>       named `dragend` at the [source
>       node](https://html.spec.whatwg.org/multipage/dnd.html#source-node).
>    3. [...]

A thorough test suite will verify that the `drop` event is fired as specified,
and it will also verify that the `dragend` event is fired as specified. An even
better test suite will also verify that the `drop` event is fired *before* the
`dragend` event.

In September of 2019, [Chromium accidentally changed the ordering of the `drop`
and `dragend`
events](https://bugs.chromium.org/p/chromium/issues/detail?id=1005747), and as
a result, real web applications stopped functioning. If there had been a test
for the sequence of these events, then this confusion would have been avoided.

When making your testing plan, be sure to look carefully for event dispatch and
the other patterns listed above. They won't always be as clear as the "drag"
example!

### Optional behavior

Specifications occasionally allow browsers discretion in how they implement
certain features. These are described using [RFC
2119](https://tools.ietf.org/html/rfc2119) terms like "MAY" and "OPTIONAL".
Although browsers should not be penalized for deciding not to implement such
behavior, WPT offers tests that verify the correctness of the browsers which
do. Be sure to [label the test as optional according to WPT's
conventions](file-names) so that people reviewing test results know how to
interpret failures.

*Example:* The algorithm underpinning
[`document.getElementsByTagName`](https://developer.mozilla.org/en-US/docs/Web/API/Document/getElementsByTagName)
includes the following paragraph:

> When invoked with the same argument, and as long as *root*'s [node
> document](https://dom.spec.whatwg.org/#concept-node-document)'s
> [type](https://dom.spec.whatwg.org/#concept-document-type) has not changed,
> the same [HTMLCollection](https://dom.spec.whatwg.org/#htmlcollection) object
> may be returned as returned by an earlier call.

That statement uses the word "may," so even though it modifies the behavior of
the preceding algorithm, it is strictly optional. The test we write for this
should be designated accordingly.

It's important to read these sections carefully because the distinction between
"mandatory" behavior and "optional" behavior can be nuanced. In this case, the
optional behavior is never allowed if the document's type has changed. That
makes for a mandatory test, one that verifies browsers don't return the same
result when the document's type changes.

## Exercising Restraint

When writing conformance tests, choosing what *not* to test is sometimes just
as hard as finding what needs testing.

### Don't dive too deep

Algorithms are composed of many other algorithms which themselves are defined
in terms of still more algorithms. It can be intimidating to consider
exhaustively testing one of those "nested" algorithms, especially when they are
shared by many different APIs.

In general, you should plan to write "surface tests" for the nested algorithms.
That means only verifying that they exhibit the basic behavior you are
expecting.

It's definitely important to test exhaustively, but it's just as important to
do so in a structured way. Reach out to the test suite's maintainers to learn
if and how they have already tested those algorithms. In many cases, it's
acceptable to test them in just one place (and maybe through a different API
entirely), and rely only on surface-level testing everywhere else. While it's
always possible for more tests to uncover new bugs, the chances may be slim.
The time we spend writing tests is highly valuable, so we have to be efficient!

*Example:* The following algorithm from [the DOM
standard](https://dom.spec.whatwg.org/) powers
[`document.querySelector`](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelector):

> To **scope-match a selectors string** *selectors* against a *node*, run these
> steps:
>
> 1. Let *s* be the result of [parse a
>    selector](https://drafts.csswg.org/selectors-4/#parse-a-selector)
>    *selectors*.
> 2. If *s* is failure, then
>    [throw](https://webidl.spec.whatwg.org/#dfn-throw) a
>    "[`SyntaxError`](https://webidl.spec.whatwg.org/#syntaxerror)"
>    [DOMException](https://webidl.spec.whatwg.org/#idl-DOMException).
> 3. Return the result of [match a selector against a
>    tree](https://drafts.csswg.org/selectors-4/#match-a-selector-against-a-tree)
>    with *s* and *node*'s
>    [root](https://dom.spec.whatwg.org/#concept-tree-root) using [scoping
>    root](https://drafts.csswg.org/selectors-4/#scoping-root) *node*.

As described earlier in this guide, we'd certainly want to test the branch
regarding the parsing failure. However, there are many ways a string might fail
to parse--should we verify them all in the tests for `document.querySelector`?
What about `document.querySelectorAll`? Should we test them all there, too?

The answers depend on the current state of the test suite: whether or not tests
for selector parsing exist and where they are located. That's why it's best to
confer with the people who are maintaining the tests.

### Avoid excessive breadth

When the set of input values is finite, it can be tempting to test them all
exhaustively. When the set is very large, test authors can reduce repetition by
defining tests programmatically in loops.

Using advanced control flow techniques to dynamically generate tests can
actually *reduce* test quality. It may obscure the intent of the tests since
readers have to mentally "unwind" the iteration to determine what is actually
being verified. The practice is more susceptible to bugs. These bugs may not be
obvious--they may not cause failures, and they may exercise fewer cases than
intended. Finally, tests authored using this approach often take a relatively
long time to complete, and that puts a burden on people who collect test
results in large numbers.

The severity of these drawbacks varies with the complexity of the generation
logic. For example, it would be pronounced in a test which conditionally made
different assertions within many nested loops. Conversely, the severity would
be low in a test which only iterated over a list of values in order to make the
same assertions about each. Recognizing when the benefits outweigh the risks
requires discretion, so once you understand them, you should use your best
judgement.

*Example:* We can see this consideration in the very first step of the
`Response` constructor from [the Fetch
standard](https://fetch.spec.whatwg.org/)

> The `Response`(*body*, *init*) constructor, when invoked, must run these
> steps:
>
> 1. If *init*["`status`"] is not in the range `200` to `599`, inclusive, then
>    [throw](https://webidl.spec.whatwg.org/#dfn-throw) a `RangeError`.
>
> [...]

This function accepts exactly 400 values for the "status." With [WPT's
testharness.js](./testharness), it's easy to dynamically create one test for
each value. Unless we have reason to believe that a browser may exhibit
drastically different behavior for any of those values (e.g. correctly
accepting `546` but incorrectly rejecting `547`), then the complexity of
testing those cases probably isn't warranted.

Instead, focus on writing declarative tests for specific values which are novel
in the context of the algorithm. For ranges like in this example, testing the
boundaries is a good idea. `200` and `599` should not produce an error while
`199` and `600` should produce an error. Feel free to use what you know about
the feature to choose additional values. In this case, HTTP response status
codes are classified by the "hundred" order of magnitude, so we might also want
to test a "3xx" value and a "4xx" value.

## Assessing coverage

It's very likely that WPT already has some tests for the feature (or at least
the specification) that you're interesting in testing. In that case, you'll
have to learn what's already been done before starting to write new tests.
Understanding the design of existing tests will let you avoid duplicating
effort, and it will also help you integrate your work more logically.

Even if the feature you're testing does *not* have any tests, you should still
keep these guidelines in mind. Sooner or later, someone else will want to
extend your work, so you ought to give them a good starting point!

### File names

The names of existing files and folders in the repository can help you find
tests that are relevant to your work. [This page on the design of
WPT](../test-suite-design) goes into detail about how files are generally laid
out in the repository.

Generally speaking, every conformance tests is stored in a subdirectory
dedicated to the specification it verifies. The structure of these
subdirectories vary. Some organize tests in directories related to algorithms
or behaviors. Others have a more "flat" layout, where all tests are listed
together.

Whatever the case, test authors try to choose names that communicate the
behavior under test, so you can use them to make an educated guess about where
your tests should go.

*Example:* Imagine you wanted to write a test to verify that headers were made
immutable by the `Request.error` method defined in [the Fetch
standard](https://fetch.spec.whatwg.org). Here's the algorithm:

> The static error() method, when invoked, must run these steps:
>
> 1. Let *r* be a new [Response](https://fetch.spec.whatwg.org/#response)
>    object, whose
>    [response](https://fetch.spec.whatwg.org/#concept-response-response) is a
>    new [network error](https://fetch.spec.whatwg.org/#concept-network-error).
> 2. Set *r*'s [headers](https://fetch.spec.whatwg.org/#response-headers) to a
>    new [Headers](https://fetch.spec.whatwg.org/#headers) object whose
>    [guard](https://fetch.spec.whatwg.org/#concept-headers-guard) is
>    "`immutable`".
> 3. Return *r*.

In order to figure out where to write the test (and whether it's needed at
all), you can review the contents of the `fetch/` directory in WPT. Here's how
that looks on a UNIX-like command line:

    $ ls fetch
    api/                           DIR_METADATA  OWNERS
    connection-pool/               h1-parsing/   private-network-access/
    content-encoding/              http-cache/   range/
    content-length/                images/       README.md
    content-type/                  metadata/     redirect-navigate/
    corb/                          META.yml      redirects/
    cross-origin-resource-policy/  nosniff/      security/
    data-urls/                     origin/       stale-while-revalidate/

This test is for a behavior directly exposed through the API, so we should look
in the `api/` directory:

    $ ls fetch/api
    abort/  cors/         headers/           policies/  request/    response/
    basic/  credentials/  idlharness.any.js  redirect/  resources/

And since this is a static method on the `Response` constructor, we would
expect the test to belong in the `response/` directory:

    $ ls fetch/api/response
    multi-globals/                   response-static-error.html
    response-cancel-stream.html      response-static-redirect.html
    response-clone.html              response-stream-disturbed-1.html
    response-consume-empty.html      response-stream-disturbed-2.html
    response-consume.html            response-stream-disturbed-3.html
    response-consume-stream.html     response-stream-disturbed-4.html
    response-error-from-stream.html  response-stream-disturbed-5.html
    response-error.html              response-stream-disturbed-6.html
    response-from-stream.any.js      response-stream-with-broken-then.any.js
    response-init-001.html           response-trailer.html
    response-init-002.html

There seems to be a test file for the `error` method:
`response-static-error.html`. We can open that to decide if the behavior is
already covered. If not, then we know where to [write the
test](https://github.com/web-platform-tests/wpt/pull/19601)!

### Failures on wpt.fyi

There are many behaviors that are difficult to describe in a succinct file
name. That's commonly the case with low-level rendering details of CSS
specifications. Test authors may resort to generic number-based naming schemes
for their files, e.g. `feature-001.html`, `feature-002.html`, etc. This makes
it difficult to determine if a test case exists judging only by the names of
files.

If the behavior you want to test is demonstrated by some browsers but not by
others, you may be able to use the *results* of the tests to locate the
relevant test.

[wpt.fyi](https://wpt.fyi) is a website which publishes results of WPT in
various browsers. Because most browsers pass most tests, the pass/fail
characteristics of the behavior you're testing can help you filter through a
large number of highly similar tests.

*Example:* Imagine you've found a bug in the way Safari renders the top CSS
border of HTML tables. By searching through directory names and file names,
you've determined the probable location for the test: the `css/CSS2/borders/`
directory. However, there are *three hundred* files that begin with
`border-top-`! None of the names mention the `<table>` element, so any one of
the files may already be testing the case you found.

Luckily, you also know that Firefox and Chrome do not exhibit this bug. You
could find such tests by visual inspection of the [wpt.fyi](https://wpt.fyi)
results overview, but [the website's "search" feature includes operators that
let you query for this information
directly](https://github.com/web-platform-tests/wpt.fyi/blob/master/api/query/README.md).
To find the tests which begin with `border-top-`, pass in Chrome, pass in
Firefox, and fail in Safari, you could write [`border-top- chrome:pass
firefox:pass
safari:fail](https://wpt.fyi/results/?label=master&label=experimental&aligned&q=border-top-%20safari%3Afail%20firefox%3Apass%20chrome%3Apass).
The results show only three such tests exist:

- `border-top-applies-to-005.xht`
- `border-top-color-applies-to-005.xht`
- `border-top-width-applies-to-005.xht`

These may not describe the behavior you're interested in testing; the only way
to know for sure is to review their contents. However, this is a much more
manageable set to work with!

### Querying file contents

Some web platform features are enabled with a predictable pattern. For example,
HTML attributes follow a fairly consistent format. If you're interested in
testing a feature like this, you may be able to learn where your tests belong
by querying the contents of the files in WPT.

You may be able to perform such a search on the web. WPT is hosted on
GitHub.com, and [GitHub offers some basic functionality for querying
code](https://help.github.com/en/articles/about-searching-on-github). If your
search criteria are short and distinctive (e.g. all files containing
"querySelectorAll"), then this interface may be sufficient. However, more
complicated criteria may require [regular
expressions](https://www.regular-expressions.info/). For that, you can
[download the WPT
repository](https://web-platform-tests.org/writing-tests/github-intro.html) and
use [git](https://git-scm.com) to perform more powerful searches.

The following table lists some common search criteria and examples of how they
can be expressed using regular expressions:

<div class="table-container">

```eval_rst
================================= ================== ==========================
Criteria                          Example match      Example regular expression
================================= ================== ==========================
JavaScript identifier references  ``obj.foo()``      ``\bfoo\b``
JavaScript string literals        ``x = "foo";``     ``(["'])foo\1``
HTML tag names                    ``<foo attr>``     ``<foo(\s|>|$)``
HTML attributes                   ``<div foo=3>``    ``<[a-zA-Z][^>]*\sfoo(\s|>|=|$)``
CSS property name                 ``style="foo: 4"`` ``([{;=\"']|\s|^)foo\s+:``
================================= ================== ==========================
```

</div>

Bear in mind that searches like this are not necessarily exhaustive. Depending
on the feature, it may be difficult (or even impossible) to write a query that
correctly identifies all relevant tests. This strategy can give a helpful
guide, but the results may not be conclusive.

*Example:* Imagine you're interested in testing how the `src` attribute of the
`iframe` element works with `javascript:` URLs. Judging only from the names of
directories, you've found a lot of potential locations for such a test. You
also know many tests use `javascript:` URLs without describing that in their
name. How can you find where to contribute new tests?

You can design a regular expression that matches many cases where a
`javascript:` URL is assigned to the `src` property in HTML. You can use the
`git grep` command to query the contents of the `html/` directory:

    $ git grep -lE "src\s*=\s*[\"']?javascript:" html
    html/browsers/browsing-the-web/navigating-across-documents/javascript-url-query-fragment-components.html
    html/browsers/browsing-the-web/navigating-across-documents/javascript-url-return-value-handling.html
    html/dom/documents/dom-tree-accessors/Document.currentScript.html
    html/dom/self-origin.sub.html
    html/editing/dnd/target-origin/114-manual.html
    html/semantics/embedded-content/media-elements/track/track-element/cloneNode.html
    html/semantics/scripting-1/the-script-element/execution-timing/040.html
    html/semantics/scripting-1/the-script-element/execution-timing/080.html
    html/semantics/scripting-1/the-script-element/execution-timing/108.html
    html/semantics/scripting-1/the-script-element/execution-timing/109.html
    html/webappapis/dynamic-markup-insertion/opening-the-input-stream/document-open-cancels-javascript-url-navigation.html

You will still have to review the contents to know which are relevant for your
purposes (if any), but compared to the 5,000 files in the `html/` directory,
this list is far more approachable!

## Writing the Tests

With a complete testing plan in hand, you now have a good idea of the scope of
your work. It's finally time to write the tests! There's a lot to say about how
this is done technically. To learn more, check out [the WPT "reftest"
tutorial](./reftest-tutorial) and [the testharness.js
tutorial](./testharness-tutorial).
