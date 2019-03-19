# Servo's style system overview

This needs to be filled more extensively. Meanwhile, you can also take a look at
the [style doc comments][style-doc], or the [Styling
Overview][wiki-styling-overview] in the wiki, a conversation between
Boris Zbarsky and Patrick Walton about how style sharing works.

<a name="selector-impl"></a>
## Selector Implementation

In order to be sharable and compatible with [Stylo][stylo](a project that aims 
to integrate Servo's style system into Gecko), the style must be consistent.

The consistency is implemented in [selectors' SelectorImpl][selector-impl], 
containing the logic related to parsing pseudo-elements and other pseudo-classes 
apart from [tree-structural ones][tree-structural-pseudo-classes].

Servo extends the selector implementation trait in order to allow a few more
things to be shared between Stylo and Servo.

The main Servo implementation (the one that is used in regular builds) is
[SelectorImpl][servo-selector-impl].

<a name="dom-glue"></a>
## DOM glue

In order to keep DOM, layout and style in different modules, there are a few
traits involved.

Style's [`dom` traits][style-dom-traits] (`TDocument`, `TElement`, `TNode`,
`TRestyleDamage`) are the main "wall" between layout and style.

Layout's [`wrapper`][layout-wrapper] module makes sure that
layout traits have the required traits implemented.

<a name="stylist"></a>
## The Stylist

The [`stylist`][stylist] structure holds all the selectors and
device characteristics for a given document.

The stylesheets' CSS rules are converted into [`Rule`][selectors-rule]s.
They are then introduced in a [`SelectorMap`][selectors-selectormap] depending
on the pseudo-element (see [`PerPseudoElementSelectorMap`][per-pseudo-selectormap]),
stylesheet origin (see [`PerOriginSelectorMap`][per-origin-selectormap]), and
priority (see the `normal` and `important` fields in
[`PerOriginSelectorMap`][per-origin-selectormap]).

This structure is effectively created once per [pipeline][docs-pipeline], in the
corresponding LayoutThread.

<a name="properties"></a>
## The `properties` module

The [properties module][properties-module] is a mako template. Its complexity is derived 
from the code that stores properties, [`cascade` function][properties-cascade-fn] and computation logic of the returned value which is exposed in the main function.

<a name="pseudo-elements"></a>
## Pseudo-Element resolution

Pseudo-elements are a tricky section of the style system. Not all
pseudo-elements are very common, and so some of them might want to skip the
cascade.

Servo has, as of right now, five [pseudo-elements][servo-pseudo-elements]:

 * [`::before`][mdn-pseudo-before] and [`::after`][mdn-pseudo-after].
 * [`::selection`][mdn-pseudo-selection]: This one is only partially
     implemented, and only works for text inputs and textareas as of right now.
 * `::-servo-details-summary`: This pseudo-element represents the `<summary>` of
     a `<details>` element.
 * `::-servo-details-content`: This pseudo-element represents the contents of
     a `<details>` element.

Both `::-servo-details-*` pseudo-elements are private (i.e. they are only parsed
from User-Agent stylesheets).

Servo has three different ways of cascading a pseudo-element, which are defined
in [`PseudoElementCascadeType`][pseudo-cascade-type]:

<a name="pe-cascading-eager"></a>
### "Eager" cascading

This mode computes the computed values of a given node's pseudo-element over the
first pass of the style system.

This is used for all public pseudo-elements, and is, as of right now, **the only
way a public pseudo-element should be cascaded** (the explanation for this is
below).

<a name="pe-cascading-precomputed"></a>
### "Precomputed" cascading

Or, better said, no cascading at all. A pseudo-element marked as such is not
cascaded.

The only rules that apply to the styles of that pseudo-element are universal
rules (rules with a `*|*` selector), and they are applied directly over the
element's style if present.

`::-servo-details-content` is an example of this kind of pseudo-element, all the
rules in the UA stylesheet with the selector `*|*::-servo-details-content` (and
only those) are evaluated over the element's style (except the `display` value,
that is overwritten by layout).

This should be the **preferred type for private pseudo-elements** (although some
of them might need selectors, see below).

<a name="pe-cascading-lazy"></a>
### "Lazy" cascading

Lazy cascading allows to compute pseudo-element styles lazily, that is, just
when needed.

Currently (for Servo, not that much for stylo), **selectors supported for this
kind of pseudo-elements are only a subset of selectors that can be matched on
the layout tree, which does not hold all data from the DOM tree**.

This subset includes tags and attribute selectors, enough for making
`::-servo-details-summary` a lazy pseudo-element (that only needs to know
if it is in an `open` details element or not).

Since no other selectors would apply to it, **this is (at least for now) not an
acceptable type for public pseudo-elements, but should be considered for private
pseudo-elements**.

#### Not found what you were looking for?

Feel free to ping @SimonSapin, @mbrubeck or @emilio on irc, and please mention
that you didn't find it here so it can be added :)

[style-doc]: https://doc.servo.org/style/index.html
[wiki-styling-overview]: https://github.com/servo/servo/wiki/Styling-overview
[stylo]: https://public.etherpad-mozilla.org/p/stylo
[selector-impl]: https://doc.servo.org/selectors/parser/trait.SelectorImpl.html
[selector-impl-ext]: https://doc.servo.org/style/selector_parser/trait.SelectorImplExt.html
[servo-selector-impl]: https://doc.servo.org/style/servo/selector_parser/struct.SelectorImpl.html
[tree-structural-pseudo-classes]: https://www.w3.org/TR/selectors4/#structural-pseudos
[style-dom-traits]: https://doc.servo.org/style/dom/index.html
[layout-wrapper]: https://doc.servo.org/layout/wrapper/index.html
[pseudo-cascade-type]: https://doc.servo.org/style/selector_parser/enum.PseudoElementCascadeType.html
[servo-pseudo-elements]: https://doc.servo.org/style/selector_parser/enum.PseudoElement.html
[mdn-pseudo-before]: https://developer.mozilla.org/en/docs/Web/CSS/::before
[mdn-pseudo-after]: https://developer.mozilla.org/en/docs/Web/CSS/::after
[mdn-pseudo-selection]: https://developer.mozilla.org/en/docs/Web/CSS/::selection
[stylist]: https://doc.servo.org/style/stylist/struct.Stylist.html
[selectors-selectormap]: https://doc.servo.org/style/selector_map/struct.SelectorMap.html
[selectors-rule]: https://doc.servo.org/style/stylist/struct.Rule.html
[per-pseudo-selectormap]: https://doc.servo.org/style/selector_parser/struct.PerPseudoElementMap.html
[per-origin-selectormap]: https://doc.servo.org/style/stylist/struct.PerOriginSelectorMap.html
[docs-pipeline]: https://github.com/servo/servo/blob/master/docs/glossary.md#pipeline
[properties-module]: https://doc.servo.org/style/properties/index.html
[properties-cascade-fn]: https://doc.servo.org/style/properties/fn.cascade.html
