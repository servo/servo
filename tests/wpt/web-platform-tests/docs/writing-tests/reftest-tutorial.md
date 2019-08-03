# Writing a reftest

<!--
Note to maintainers:

This tutorial is designed to be an authentic depiction of the WPT contribution
experience. It is not intended to be comprehensive; its scope is intentionally
limited in order to demonstrate authoring a complete test without overwhelming
the reader with features. Because typical WPT usage patterns change over time,
this should be updated periodically; please weigh extensions against the
demotivating effect that a lengthy guide can have on new contributors.
-->

Let's say you've discovered that WPT doesn't have any tests for the `dir`
attribute of [the `<bdo>`
element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/bdo). This
tutorial will guide you through the process of writing and submitting a test.
You'll need to [configure your system to use WPT's
tools](../running-tests/from-local-system), but you won't need them until
towards the end of this tutorial. Although it includes some very brief
instructions on using git, you can find more guidance in [the tutorial for git
and GitHub](../appendix/github-intro).

WPT's reftests are great for testing web-platform features that have some
visual effect. [The reftests reference page](reftests) describes them in the
abstract, but for the purposes of this guide, we'll only consider the features
we need to test the `<bdo>` element.

```eval_rst
.. contents::
   :local:
```

## Setting up your workspace

To make sure you have the latest code, first type the following into a terminal
located in the root of the WPT git repository:

    $ git fetch git@github.com:web-platform-tests/wpt.git

Next, we need a place to store the change set we're about to author. Here's how
to create a new git branch named `reftest-for-bdo` from the revision of WPT we
just downloaded:

    $ git checkout -b reftest-for-bdo FETCH_HEAD

Now you're ready to create your patch.

## Writing the test file

First, we'll create a file that demonstrates the "feature under test." That is:
we'll write an HTML document that displays some text using a `<bdo>` element.

WPT has thousands of tests, so it can be daunting to decide where to put a new
one. Generally speaking, [test files should be placed in directories
corresponding to the specification text they are
verifying](../test-suite-design). `<bdo>` is defined in [the "text-level
semantics" chapter of the HTML
specification](https://html.spec.whatwg.org/multipage/text-level-semantics.html),
so we'll want to create our new test in the directory
`html/semantics/text-level-semantics/the-bdo-element/`. Create a file named
`rtl.html` and open it in your text editor.

Here's one way to demonstrate the feature:

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>BDO element dir=rtl</title>
<link rel="help" href="https://html.spec.whatwg.org/#the-bdo-element">
<meta name="assert" content="BDO element's DIR content attribute renders corrently given value of 'rtl'.">

<p>Test passes if WAS is displayed below.</p>
<bdo dir="rtl">SAW</bdo>
```

That's pretty dense! Let's break it down:

- ```html
  <!DOCTYPE html>
  <meta charset="utf-8">
  ```

  We explicitly set the DOCTYPE and character set to be sure that browsers
  don't infer them to be something we aren't expecting. We're omitting the
  `<html>` and `<head>` tags. That's a common practice in WPT, preferred
  because it makes tests more concise.

- ```html
  <title>BDO element dir=rtl</title>
  ```
  The document's title should succinctly describe the feature under test.

- ```html
  <link rel="help" href="https://html.spec.whatwg.org/#the-bdo-element">
  ```

  The "help" metadata should reference the specification under test so that
  everyone understands the motivation. This is so helpful that [the CSS Working
  Group requires it for CSS tests](css-metadata)! If you're writing a reftest
  for a feature outside of CSS, feel free to omit this tag.

- ```html
  <meta name="assert" content="BDO element's DIR content attribute renders corrently given value of 'rtl'.">
  ```

  The "assert" metadata is a structured way for you to describe exactly what
  you want your reftest to verify. For a direct test like the one we're writing
  here, it might seem a little superfluous. It's much more helpful for
  more-involved tests where reviewers might need some help understanding your
  intentions.

  This tag is optional, so you can skip it if you think it's unnecessary. We
  recommend using it for your first few tests since it may let reviewers give
  you more helpful feedback. As you get more familiar with WPT and the
  specifications, you'll get a sense for when and where it's better to leave it
  out.

- ```html
  <p>Test passes if WAS is displayed below.</p>
  ```

  We're communicating the "pass" condition in plain English to make the test
  self-describing.

- ```html
  <bdo dir="rtl">SAW</bdo>
  ```

  This is the real focus of the test. We're including some text inside a
  `<bdo>` element in order to demonstrate the feature under test.

Since this page doesn't rely on any [special WPT server
features](server-features), we can view it by loading the HTML file directly.
There are a bunch of ways to do this; one is to navigate to the
`html/semantics/text-level-semantics/the-bdo-element/` directory in a file
browser and drag the new `rtl.html` file into an open web browser window.

![](/assets/reftest-tutorial-test-screenshot.png "screen shot of the new test")

Sighted people can open that document and verify whether or not the stated
expectation is satisfied. If we were writing a [manual test](manual), we'd be
done. However, it's time-consuming for a human to run tests, so we should
prefer making tests automatic whenever possible. Remember that we set out to
write a "reference test." Now it's time to write the reference file.

## Writing a "match" reference

The "match" reference file describes what the test file is supposed to look
like. Critically, it *must not* use the technology that we are testing. The
reference file is what allows the test to be run by a computer--the computer
can verify that each pixel in the test document exactly matches the
corresponding pixel in the reference document.

Make a new file in the same
`html/semantics/text-level-semantics/the-bdo-element/` directory named
`rtl-ref.html`, and save the following markup into it:

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>BDO element dir=rtl reference</title>

<p>Test passes if WAS is displayed below.</p>
<p>WAS</p>
```

This is like a stripped-down version of the test file. In order to produce a
visual rendering which is the same as the expected rendering, it uses a `<p>`
element whose contents is the characters in right-to-left order. That way, if
the browser doesn't support the `<bdo>` element, this file will still show text
in the correct sequence.

This file is also completely functional without the WPT server, so you can open
it in a browser directly from your hard drive.

Currently, there's no way for a human operator or an automated script to know
that the two files we've created are supposed to match visually. We'll need to
add one more piece of metadata to the test file we created earlier. Open
`html/semantics/text-level-semantics/the-bdo-element/rtl.html` in your text
editor and add another `<link>` tag as described by the following change
summary:

```diff
 <!DOCTYPE html>
 <meta charset="utf-8">
 <title>BDO element dir=rtl</title>
 <link rel="author" title="Sam Smith" href="mailto:sam@example.com">
 <link rel="help" href="https://html.spec.whatwg.org/#the-bdo-element">
+<link rel="match" href="rtl-ref.html">
 <meta name="assert" content="BDO element's DIR content attribute renders corrently given value of 'rtl'.">

 <p>Test passes if WAS is displayed below.</p>
 <bdo dir="rtl">SAW</bdo>
```

Now, anyone (human or computer) reviewing the test file will know where to find
the associated reference file.

## Verifying our work

We're done writing the test, but we should make sure it fits in with the rest
of WPT before we submit it. This involves using some of the project's tools, so
this is the point you'll need to [configure your system to run
WPT](../running-tests/from-local-system).

[The lint tool](lint-tool) can detect some of the common mistakes people make
when contributing to WPT. To run it, open a command-line terminal, navigate to
the root of the WPT repository, and enter the following command:

    python ./wpt lint html/semantics/text-level-semantics/the-bdo-element

If this recognizes any of those common mistakes in the new files, it will tell
you where they are and how to fix them. If you do have changes to make, you can
run the command again to make sure you got them right.

Now, we'll run the test using the automated pixel-by-pixel comparison approach
mentioned earlier. This is important for reftests because the test and the
reference may differ in very subtle ways that are hard to catch with the naked
eye. That's not to say your test has to pass in all browsers (or even in *any*
browser). But if we expect the test to pass, then running it this way will help
us catch other kinds of mistakes.

The tools support running the tests in many different browsers. We'll use
Firefox this time:

    python ./wpt run firefox html/semantics/text-level-semantics/the-bdo-element/rtl.html

We expect this test to pass, so if it does, we're ready to submit it. If we
were testing a web platform feature that Firefox didn't support, we would
expect the test to fail instead.

There are a few problems to look out for in addition to passing/failing status.
The report will describe fewer tests than we expect if the test isn't run at
all. That's usually a sign of a formatting mistake, so you'll want to make sure
you've used the right file names and metadata. Separately, the web browser
might crash. That's often a sign of a browser bug, so you should consider
[reporting it to the browser's
maintainers](https://rachelandrew.co.uk/archives/2017/01/30/reporting-browser-bugs/)!

## Submitting the test

First, let's stage the new files for committing:

    $ git add html/semantics/text-level-semantics/the-bdo-element/rtl.html
    $ git add html/semantics/text-level-semantics/the-bdo-element/rtl-ref.html

We can make sure the commit has everything we want to submit (and nothing we
don't) by using `git diff`:

    $ git diff --staged

On most systems, you can use the arrow keys to navigate through the changes,
and you can press the `q` key when you're done reviewing.

Next, we'll create a commit with the staged changes:

    $ git commit -m '[html] Add test for the `<bdo>` element'

And now we can push the commit to our fork of WPT:

    $ git push origin reftest-for-bdo

The last step is to submit the test for review. WPT doesn't actually need the
test we wrote in this tutorial, but if we wanted to submit it for inclusion in
the repository, we would create a pull request on GitHub. [The guide on git and
GitHub](../appendix/github-intro) has all the details on how to do that.

## More practice

Here are some ways you can keep experimenting with WPT using this test:

- Improve coverage by adding more tests for related behaviors (e.g. nested
  `<bdo>` elements)
- Add another reference document which describes what the test should *not*
  look like using [`rel=mismatch`](reftests)
