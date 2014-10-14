Scrolling
=========

Scrolling is implemented by the compositor. Compositor layers that opt in to
scroll events via the `wants_scroll_events` flag can scroll their contents.
These will be referred "scrolling roots." Scrolling roots serve as a viewport
into their content, which is stored in descendant layers. In order for
scrolling roots to be able to scroll their content, they need to be smaller
than that content. If the content was smaller than the scrolling root, it would
not be able to move around inside the scrolling root. Imagine a browser window
that is larger than the content that it contains. The size of each layer is
defined by the window size (the root layer) or the block size for iframes and
elements with `overflow:scroll`.

Since the compositor allows layers to exist independently of their parents,
child layers can overflow or fail to intersect their parents completely. To
prevent this, scrolling roots use the `masks_to_bounds` flag, which is a signal
to the compositor that it should not paint the parts of descendant layers that
lie outside the boundaries of the scrolling root.

Below is an ASCII art diagram showing a scrolling root with three content
layers (a, b, and c), scrolled down a few ticks. `masks_to_bounds` has not been
applied in the diagram.

<pre>
+-----------------------+
|                       |
=========================
|                       |  scrolling
|           &lt;-------------+root
|                       |
|             +-------+ |
=========================
|             |  b    | |
++-------+    +--^----+ |
||       |       |      |
||       |       |      |  content
||   c &lt;---------+---------+layers
|+-------+    /         |
|          a &lt;          |
|                       |
+-----------------------+
</pre>

Everything above and below the set of `====` bars would be hidden by
`masks_to_bounds`, so the composited scene will just be the viewport defined by
the scrolling root with the content layers a and b visible.

<pre>
=========================
|                       |
|                       |
|                       |
|             +-------+ |
=========================
</pre>

