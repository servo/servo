---
layout: page
title: Test Assumptions
order: 13
---

The tests make a number of assumptions of the user agent, and new
tests can freely rely on these assumptions being true:

 * The device is a full-color device.
 * The device has a viewport width of at least 800px.
 * The UA imposes no minimum font size.
 * The `medium` `font-size` computes to 16px.
 * The canvas background is `white`.
 * The initial value of `color` is `black`.
 * The user stylesheet is empty (except where indicated by the tests).
 * The device is interactive and uses scroll bars.
 * The device has the Ahem font installed.
 * The HTML `div` element is assigned `display: block;`, the
   `unicode-bidi` property may be declared, and no other property
   declarations.
   <!-- unicode-bidi: isolate should be required; we currently don't
   assume this because Chrome and Safari are yet to ship this: see
   https://bugs.chromium.org/p/chromium/issues/detail?id=296863 and
   https://bugs.webkit.org/show_bug.cgi?id=65617 -->
 * The HTML `span` element is assigned `display: inline;` and no other
   property declaration.
 * The HTML `p` element is assigned `display: block;`
 * The HTML `li` element is assigned `display: list-item;`
 * The HTML `table` elements `table`, `tbody`, `tr`, and `td` are
   assigned the `display` values `table`, `table-row-group`,
   `table-row`, and `table-cell`, respectively.
 * The UA implements reasonable line-breaking behavior; e.g., it is
   assumed that spaces between alphanumeric characters provide line
   breaking opportunities and that UAs will not break at every
   opportunity, but only near the end of a line unless a line break is
   forced.

Tests for printing behavior make some further assumptions:

 * The UA is set to print background colors and, if it supports
   graphics, background images.
 * The UA implements reasonable page-breaking behavior; e.g., it is
   assumed that UAs will not break at every opportunity, but only near
   the end of a page unless a page break is forced.
