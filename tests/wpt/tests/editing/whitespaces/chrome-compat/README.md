# Tests under `editing/whitespaces/chrome-compat` #

The main purpose of the tests under this directory is to check how the browsers
normalize collapsible white-space sequence compatible with Chrome.  So, all
tests should be updated when they start failing on Chrome.

In other words, these tests do **NOT** suggest ideal behavior.  So, nobody
should consider which browser is the best/better/worse/worst one from the
score.  However, you can check how compatibility between browsers and the
browser vendors can refer what difference causes web-compat issues in the wild.

## Basic format of a white-space sequence ##

Basically, Chrome starts a white-space sequence with an NBSP and ends a
white-space sequence with an NBSP if the white-space ends at the end of the
`Text` node.  Then, repeat the pair of an ASCII white-space and an NBSP until
reaching following visible character or an NBSP at the end of the `Text` node.

## The range to normalize white-spaces when updating a `Text` node ##

When modifying a `Text`, all white-space sequence in inserting text and
adjacent white-space sequence of deleting range boundaries should be normalized.

## The range to normalize white-spaces when deleting adjacent content of `Text` ##

If there is a following `Text` node which starts with white-spaces, the
white-space sequence should be normalized.  However, Chrome does not touch
the preceding `Text` node of the deleting content.

## The range to normalize white-spaces when joining `Text` nodes ##

If the following `Text` node starts with white-spaces, the white-space sequence
should be normalized.  However, Chrome does not touch the preceding `Text` even
if it ends with white-spaces.

## The range to normalize white-spaces when splitting a `Text` node ##

If the left node ends with a white-space, it needs to end with an NBSP.
However, Chrome does nothing for the preceding white-spaces when it ends with an
NBSP.

On the other hand, if the right node starts with a white-space, the white-space
sequence should be normalized and should start with an NBSP.
