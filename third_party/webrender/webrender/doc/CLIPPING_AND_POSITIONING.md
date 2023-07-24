# Original Design

To understand the current design for clipping and positioning (transformations
and scrolling) in WebRender it can be useful to have a little background about
the original design for these features. The most important thing to remember is
that originally clipping, scrolling regions, and transformations were
properties of stacking contexts and they were completely _hierarchical_. This
goes a long way toward representing the majority of CSS content on the web, but
fails when dealing with important edges cases and features including:
 1. Support for sticky positioned content
 2. Scrolling areas that include content that is ordered both above and below
    intersecting content from outside the scroll area.
 3. Items in the same scrolling root, clipped by different clips one or more of
    which are defined outside the scrolling root itself.
 4. Completely non-hierarchical clipping situations, such as when items are
    clipped by some clips in the hierarchy, but not others.

Design changes have been a step by step path from the original design to one
that can handle all CSS content.

# Current Design

All positioning and clipping is handled by the `SpatialTree`. The name is a
holdover from when this tree was a tree of `Layers` which handled both
positioning and clipping. Currently the `SpatialTree` holds:
 1. A hierarchical collection of `SpatialNodes`, with the final screen
    transformation of each node depending on the relative transformation of the
    node combined with the transformations of all of its ancestors. These nodes
    are responsible for positioning display list items and clips.
 2. A collection of `ClipNodes` which specify a rectangular clip and, optionally,
    a set of rounded rectangle clips and a masking image.
 3. A collection of `ClipChains`. Each `ClipChain` is a list of `ClipNode`
    elements. Every display list item has an assigned `ClipChain` which
    specifies what `ClipNodes` are applied to that item.

The `SpatialNode` of each clip applied to an item is completely independent of
the `SpatialNode` applied to the item itself.

One holdover from the previous design is that both `ClipNode` and `SpatialNodes`
have a parent node, which is either a `SpatialNode` or a `ClipNode`.  From this
node WebRender can determine both a parent `ClipNode` and a parent `SpatialNode`
by finding the first ancestor of that type. This is handled by the
`DisplayListFlattener`.

## `SpatialNode`
There are three types of `SpatialNodes`:
  1. Reference frames which are created when content needs to apply
     transformation or perspective properties to display list items. Reference
     frames establish a new coordinate system, so internally all coordinates on
     display list items are relative to the reference frame origin. Later
     any non-reference frame positioning nodes that display list items belong
     to can adjust this position relative to the reference frame origin.
  2. Scrolling nodes are used to define scrolling areas. These nodes have scroll
     offsets which are a 2D translation relative to ancestor nodes and, ultimately,
     the reference frame origin.
  3. Sticky frames are responsible for implementing position:sticky behavior.
     This is also an 2D translation.

`SpatialNodes` are defined as items in the display list. After scene building
each node is traversed hierarchically during the `SpatialTree::update()` step.
Once reference frame transforms and relative offsets are calculated, a to screen
space transformation can be calculated for each `SpatialNode`. This transformation
is added the `TransformPalette` and becomes directly available to WebRender shaders.

In addition to screen space transformation calculation, the `SpatialNode` tree
is divided up into _compatible coordinate systems_. These are coordinate systems
which differ only by 2D translations from their parent system. These compatible
coordinate systems may even cross reference frame boundaries. The goal here is
to allow the application clipping rectangles from different compatible
coordinate systems without generating mask images.

## `ClipNode`

Each clip node holds a clip rectangle along with an optional collection of
rounded clip rectangles and a mask image. The fact that `ClipNodes` all have a
clip rectangle is important because it means that all content clipped by a
clip node has a bounding rectangle, which can be converted into a bounding
screen space rectangle.  This rectangle is called the _outer rectangle_ of the
clip. `ClipNodes` may also have an _inner rectangle_, which is an area within
the boundaries of the _outer rectangle_ that is completely unclipped.

These rectangles are calculated during the `SpatialTree::update()` phase. In
addition, each `ClipNode` produces a template `ClipChainNode` used to build
the `ClipChains` which use that node.

## `ClipChains`

There are two ways that `ClipChains` are defined in WebRender. The first is
through using the API for manually specifying `ClipChains` via a parent
`ClipChain` and a list of `ClipNodes`. The second is through the hierarchy of a
`ClipNode` established by its parent node. Every `ClipNode` has a chain of
ancestor `SpatialNodes` and `ClipNodes`. The creation of a `ClipNode`
automatically defines a `ClipChain` for this hierarchy. This behavior is a
compatibility feature with the old completely hierarchical clipping architecture
and is still how Gecko and Servo create most of their `ClipChains`. These
hierarchical `ClipChains` are constructed during the `ClipNode::update()` step.

During `ClipChain` construction, WebRender tries to eliminate clips that will
not affect rendering, by looking at the combined _outer rectangle_ and _inner
rectangle_ of a `ClipChain` and the _outer rectangle_ and _inner rectangle_ of
any `ClipNode` appended to the chain. An example of the goal of this process is
to avoid having to render a mask for a large rounded rectangle when the rest of
the clip chain constrains the content to an area completely inside that
rectangle. Avoiding mask rasterization in this case and others has large
performance impacts on WebRender.

# Clipping and Positioning in the Display List

Each non-structural WebRender display list item has
 * A `SpatialId` of a `SpatialNode` for positioning
 * A `ClipId` of a `ClipNode` or a `ClipChain` for clipping
 * An item-specific rectangular clip rectangle

The positioning node determines how that item is positioned. It's assumed that
the positioning node and the item are children of the same reference frame. The
clipping node determines how that item is clipped. This should be fully
independent of how the node is positioned and items can be clipped by any
`ClipChain` regardless of the reference frame of their member clips. Finally,
the item-specific clipping rectangle is applied directly to the item and should
never result in the creation of a clip mask itself.

## Converting user-exposed `ClipId`/`SpatialId` to internal indices

WebRender must access `ClipNodes` and `SpatialNodes` quite a bit when building
scenes and frames, so it tries to convert `ClipId`/`SpatialId`, which are already
per-pipeline indices, to global scene-wide indices.  Internally this is a
conversion from `ClipId` into `ClipNodeIndex` or `ClipChainIndex`, and from
`SpatialId` into `SpatialNodeIndex`. In order to make this conversion cheaper, the
`DisplayListFlattner` assigns offsets for each pipeline and node type in the
scene-wide `SpatialTree`.

Nodes are added to their respective arrays sequentially as the display list is
processed during scene building. When encountering an iframe, the
`DisplayListFlattener` must start processing the nodes for that iframe's
pipeline, meaning that nodes are now being added out of order to the node arrays
of the `SpatialTree`. In this case, the `SpatialTree` fills in the gaps in
the node arrays with placeholder nodes.

# Hit Testing

Hit testing is the responsibility of the `HitTester` data structure. This
structure copies information necessary for hit testing from the
`SpatialTree`. This is done so that hit testing can still take place while a
new `SpatialTree` is under construction.

# Ideas for the Future
1. Expose the difference between `ClipId` and `ClipChainId` in the API.
2. Prevent having to duplicate the `SpatialTree` for hit testing.
3. Avoid having to create placeholder nodes in the `SpatialTree` while
   processing iframes.
