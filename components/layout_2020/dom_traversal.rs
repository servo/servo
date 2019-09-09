use super::*;
use crate::dom::{Document, NodeData, NodeId};
use crate::style::StyleSet;
use atomic_refcell::AtomicRefMut;

pub(super) struct Context<'a> {
    pub document: &'a Document,
    pub author_styles: &'a StyleSet,
}

#[derive(Copy, Clone)]
pub(super) enum WhichPseudoElement {
    Before,
    After,
}

pub(super) enum Contents {
    /// Refers to a DOM subtree, plus `::before` and `::after` pseudo-elements.
    OfElement(NodeId),

    /// Example: an `<img src=…>` element.
    /// <https://drafts.csswg.org/css2/conform.html#replaced-element>
    Replaced(ReplacedContent),

    /// Content of a `::before` or `::after` pseudo-element this is being generated.
    /// <https://drafts.csswg.org/css2/generate.html#content>
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum NonReplacedContents {
    OfElement(NodeId),
    OfPseudoElement(Vec<PseudoElementContentItem>),
}

pub(super) enum PseudoElementContentItem {
    Text(String),
    Replaced(ReplacedContent),
}

pub(super) trait TraversalHandler<'dom> {
    fn handle_text(&mut self, text: &str, parent_style: &Arc<ComputedValues>);

    /// Or pseudo-element
    fn handle_element(
        &mut self,
        style: &Arc<ComputedValues>,
        display: DisplayGeneratingBox,
        contents: Contents,
        box_slot: BoxSlot<'dom>,
    );
}

fn traverse_children_of<'dom>(
    parent_element: NodeId,
    parent_element_style: &Arc<ComputedValues>,
    context: &'dom Context,
    handler: &mut impl TraversalHandler<'dom>,
) {
    traverse_pseudo_element(
        WhichPseudoElement::Before,
        parent_element,
        parent_element_style,
        context,
        handler,
    );

    let mut next = context.document[parent_element].first_child;
    while let Some(child) = next {
        match &context.document[child].data {
            NodeData::Document
            | NodeData::Doctype { .. }
            | NodeData::Comment { .. }
            | NodeData::ProcessingInstruction { .. } => {}
            NodeData::Text { contents } => {
                handler.handle_text(contents, parent_element_style);
            }
            NodeData::Element(_) => traverse_element(child, parent_element_style, context, handler),
        }
        next = context.document[child].next_sibling
    }

    traverse_pseudo_element(
        WhichPseudoElement::After,
        parent_element,
        &parent_element_style,
        context,
        handler,
    );
}

fn traverse_element<'dom>(
    element_id: NodeId,
    parent_element_style: &ComputedValues,
    context: &'dom Context,
    handler: &mut impl TraversalHandler<'dom>,
) {
    let style = style_for_element(
        context.author_styles,
        context.document,
        element_id,
        Some(parent_element_style),
    );
    match style.box_.display {
        Display::None => context.unset_boxes_in_subtree(element_id),
        Display::Contents => {
            if ReplacedContent::for_element(element_id, context).is_some() {
                // `display: content` on a replaced element computes to `display: none`
                // <https://drafts.csswg.org/css-display-3/#valdef-display-contents>
                context.unset_boxes_in_subtree(element_id)
            } else {
                context.layout_data_mut(element_id).self_box = Some(LayoutBox::DisplayContents);
                traverse_children_of(element_id, &style, context, handler)
            }
        }
        Display::GeneratingBox(display) => handler.handle_element(
            &style,
            display,
            match ReplacedContent::for_element(element_id, context) {
                Some(replaced) => Contents::Replaced(replaced),
                None => Contents::OfElement(element_id),
            },
            context.element_box_slot(element_id),
        ),
    }
}

fn traverse_pseudo_element<'dom>(
    which: WhichPseudoElement,
    element: NodeId,
    element_style: &ComputedValues,
    context: &'dom Context,
    handler: &mut impl TraversalHandler<'dom>,
) {
    if let Some(style) = pseudo_element_style(which, element, element_style, context) {
        match style.box_.display {
            Display::None => context.unset_pseudo_element_box(element, which),
            Display::Contents => {
                context.unset_pseudo_element_box(element, which);
                let items = generate_pseudo_element_content(&style, element, context);
                traverse_pseudo_element_contents(&style, items, handler);
            }
            Display::GeneratingBox(display) => {
                let items = generate_pseudo_element_content(&style, element, context);
                let contents = Contents::OfPseudoElement(items);
                let box_slot = context.pseudo_element_box_slot(element, which);
                handler.handle_element(&style, display, contents, box_slot);
            }
        }
    }
}

fn traverse_pseudo_element_contents<'dom>(
    pseudo_element_style: &Arc<ComputedValues>,
    items: Vec<PseudoElementContentItem>,
    handler: &mut impl TraversalHandler<'dom>,
) {
    let mut anonymous_style = None;
    for item in items {
        match item {
            PseudoElementContentItem::Text(text) => {
                handler.handle_text(&text, pseudo_element_style)
            }
            PseudoElementContentItem::Replaced(contents) => {
                let item_style = anonymous_style.get_or_insert_with(|| {
                    ComputedValues::anonymous_inheriting_from(Some(pseudo_element_style))
                });
                let display_inline = DisplayGeneratingBox::OutsideInside {
                    outside: DisplayOutside::Inline,
                    inside: DisplayInside::Flow,
                };
                // `display` is not inherited, so we get the initial value
                debug_assert!(item_style.box_.display == Display::GeneratingBox(display_inline));
                handler.handle_element(
                    item_style,
                    display_inline,
                    Contents::Replaced(contents),
                    // We don’t keep pointers to boxes generated by contents of pseudo-elements
                    BoxSlot::dummy(),
                )
            }
        }
    }
}

impl std::convert::TryFrom<Contents> for NonReplacedContents {
    type Error = ReplacedContent;

    fn try_from(contents: Contents) -> Result<Self, Self::Error> {
        match contents {
            Contents::OfElement(id) => Ok(NonReplacedContents::OfElement(id)),
            Contents::OfPseudoElement(items) => Ok(NonReplacedContents::OfPseudoElement(items)),
            Contents::Replaced(replaced) => Err(replaced),
        }
    }
}

impl std::convert::From<NonReplacedContents> for Contents {
    fn from(contents: NonReplacedContents) -> Self {
        match contents {
            NonReplacedContents::OfElement(id) => Contents::OfElement(id),
            NonReplacedContents::OfPseudoElement(items) => Contents::OfPseudoElement(items),
        }
    }
}

impl NonReplacedContents {
    pub fn traverse<'dom>(
        self,
        inherited_style: &Arc<ComputedValues>,
        context: &'dom Context,
        handler: &mut impl TraversalHandler<'dom>,
    ) {
        match self {
            NonReplacedContents::OfElement(id) => {
                traverse_children_of(id, inherited_style, context, handler)
            }
            NonReplacedContents::OfPseudoElement(items) => {
                traverse_pseudo_element_contents(inherited_style, items, handler)
            }
        }
    }
}

fn pseudo_element_style(
    _which: WhichPseudoElement,
    _element: NodeId,
    _element_style: &ComputedValues,
    _context: &Context,
) -> Option<Arc<ComputedValues>> {
    // FIXME: run the cascade, then return None for `content: normal` or `content: none`
    // https://drafts.csswg.org/css2/generate.html#content
    None
}

fn generate_pseudo_element_content(
    _pseudo_element_style: &ComputedValues,
    _element: NodeId,
    _context: &Context,
) -> Vec<PseudoElementContentItem> {
    let _ = PseudoElementContentItem::Text;
    let _ = PseudoElementContentItem::Replaced;
    unimplemented!()
}

pub(super) struct BoxSlot<'dom> {
    slot: Option<AtomicRefMut<'dom, Option<LayoutBox>>>,
}

impl<'dom> BoxSlot<'dom> {
    pub fn new(mut slot: AtomicRefMut<'dom, Option<LayoutBox>>) -> Self {
        *slot = None;
        Self { slot: Some(slot) }
    }

    pub fn dummy() -> Self {
        Self { slot: None }
    }

    pub fn set(mut self, box_: LayoutBox) {
        if let Some(slot) = &mut self.slot {
            **slot = Some(box_)
        }
    }
}

impl Drop for BoxSlot<'_> {
    fn drop(&mut self) {
        if let Some(slot) = &mut self.slot {
            assert!(slot.is_some(), "failed to set a layout box")
        }
    }
}

impl Context<'_> {
    fn layout_data_mut(&self, element_id: NodeId) -> AtomicRefMut<LayoutDataForElement> {
        self.document[element_id]
            .as_element()
            .unwrap()
            .layout_data
            .borrow_mut()
    }

    fn element_box_slot(&self, element_id: NodeId) -> BoxSlot {
        BoxSlot::new(AtomicRefMut::map(
            self.layout_data_mut(element_id),
            |data| &mut data.self_box,
        ))
    }

    fn pseudo_element_box_slot(&self, element_id: NodeId, which: WhichPseudoElement) -> BoxSlot {
        BoxSlot::new(AtomicRefMut::map(
            self.layout_data_mut(element_id),
            |data| {
                let pseudos = data.pseudo_elements.get_or_insert_with(Default::default);
                match which {
                    WhichPseudoElement::Before => &mut pseudos.before,
                    WhichPseudoElement::After => &mut pseudos.after,
                }
            },
        ))
    }

    fn unset_pseudo_element_box(&self, element_id: NodeId, which: WhichPseudoElement) {
        if let Some(pseudos) = &mut self.layout_data_mut(element_id).pseudo_elements {
            match which {
                WhichPseudoElement::Before => pseudos.before = None,
                WhichPseudoElement::After => pseudos.after = None,
            }
        }
    }

    fn unset_boxes_in_subtree(&self, base_element: NodeId) {
        let mut node_id = base_element;
        loop {
            let node = &self.document[node_id];
            if let Some(element_data) = node.as_element() {
                let mut layout_data = element_data.layout_data.borrow_mut();
                layout_data.pseudo_elements = None;
                if layout_data.self_box.take().is_some() {
                    // Only descend into children if we removed a box.
                    // If there wasn’t one, then descendants don’t have boxes either.
                    if let Some(child) = node.first_child {
                        node_id = child;
                        continue;
                    }
                }
            }
            let mut next_is_a_sibling_of = node_id;
            node_id = loop {
                if let Some(sibling) = self.document[next_is_a_sibling_of].next_sibling {
                    break sibling;
                } else {
                    next_is_a_sibling_of = node
                        .parent
                        .expect("reached the root while traversing only a subtree");
                }
            };
            if next_is_a_sibling_of == base_element {
                // Don’t go outside the subtree
                return;
            }
        }
    }
}
