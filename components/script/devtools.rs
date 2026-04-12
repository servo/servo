/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::str;

use devtools_traits::{
    AncestorData, AttrModification, AutoMargins, ComputedNodeLayout, CssDatabaseProperty,
    EventListenerInfo, MatchedRule, NodeInfo, NodeStyle, RuleModification, TimelineMarker,
    TimelineMarkerType,
};
use js::context::JSContext;
use markup5ever::{LocalName, ns};
use rustc_hash::FxHashMap;
use script_bindings::root::Dom;
use servo_base::generic_channel::GenericSender;
use servo_base::id::PipelineId;
use servo_config::pref;
use style::attr::AttrValue;

use crate::document_collection::DocumentCollection;
use crate::dom::bindings::codegen::Bindings::CSSGroupingRuleBinding::CSSGroupingRuleMethods;
use crate::dom::bindings::codegen::Bindings::CSSLayerBlockRuleBinding::CSSLayerBlockRuleMethods;
use crate::dom::bindings::codegen::Bindings::CSSRuleListBinding::CSSRuleListMethods;
use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::CSSStyleRuleBinding::CSSStyleRuleMethods;
use crate::dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use crate::dom::bindings::codegen::Bindings::DOMRectBinding::DOMRectMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeConstants;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::css::cssstyledeclaration::ENABLED_LONGHAND_PROPERTIES;
use crate::dom::css::cssstylerule::CSSStyleRule;
use crate::dom::document::AnimationFrameCallback;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::types::{CSSGroupingRule, CSSLayerBlockRule, EventTarget, HTMLElement};
use crate::realms::enter_realm;
use crate::script_runtime::CanGc;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable)]
pub(crate) struct PerPipelineState {
    #[no_trace]
    pipeline: PipelineId,

    /// Maps from a node's unique ID to the Node itself
    known_nodes: FxHashMap<String, Dom<Node>>,
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, Default)]
pub(crate) struct DevtoolsState {
    per_pipeline_state: RefCell<FxHashMap<NoTrace<PipelineId>, PerPipelineState>>,
}

impl PerPipelineState {
    fn register_node(&mut self, node: &Node) {
        let unique_id = node.unique_id(self.pipeline);
        self.known_nodes
            .entry(unique_id)
            .or_insert_with(|| Dom::from_ref(node));
    }
}

impl DevtoolsState {
    pub(crate) fn notify_pipeline_created(&self, pipeline: PipelineId) {
        self.per_pipeline_state.borrow_mut().insert(
            NoTrace(pipeline),
            PerPipelineState {
                pipeline,
                known_nodes: Default::default(),
            },
        );
    }
    pub(crate) fn notify_pipeline_exited(&self, pipeline: PipelineId) {
        self.per_pipeline_state
            .borrow_mut()
            .remove(&NoTrace(pipeline));
    }

    fn pipeline_state_for(&self, pipeline: PipelineId) -> Option<Ref<'_, PerPipelineState>> {
        Ref::filter_map(self.per_pipeline_state.borrow(), |state| {
            state.get(&NoTrace(pipeline))
        })
        .ok()
    }

    fn mut_pipeline_state_for(&self, pipeline: PipelineId) -> Option<RefMut<'_, PerPipelineState>> {
        RefMut::filter_map(self.per_pipeline_state.borrow_mut(), |state| {
            state.get_mut(&NoTrace(pipeline))
        })
        .ok()
    }

    pub(crate) fn wants_updates_for_node(&self, pipeline: PipelineId, node: &Node) -> bool {
        let Some(unique_id) = node.unique_id_if_already_present() else {
            // This node does not have a unique id, so clearly the devtools inspector
            // hasn't seen it before.
            return false;
        };
        self.pipeline_state_for(pipeline)
            .is_some_and(|pipeline_state| pipeline_state.known_nodes.contains_key(&unique_id))
    }

    fn find_node_by_unique_id(&self, pipeline: PipelineId, node_id: &str) -> Option<DomRoot<Node>> {
        self.pipeline_state_for(pipeline)?
            .known_nodes
            .get(node_id)
            .map(|node: &Dom<Node>| node.as_rooted())
    }
}

pub(crate) fn handle_set_timeline_markers(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    marker_types: Vec<TimelineMarkerType>,
    reply: GenericSender<Option<TimelineMarker>>,
) {
    match documents.find_window(pipeline) {
        None => reply.send(None).unwrap(),
        Some(window) => window.set_devtools_timeline_markers(marker_types, reply),
    }
}

pub(crate) fn handle_drop_timeline_markers(
    documents: &DocumentCollection,
    pipeline: PipelineId,
    marker_types: Vec<TimelineMarkerType>,
) {
    if let Some(window) = documents.find_window(pipeline) {
        window.drop_devtools_timeline_markers(marker_types);
    }
}

pub(crate) fn handle_request_animation_frame(
    documents: &DocumentCollection,
    id: PipelineId,
    actor_name: String,
) {
    if let Some(doc) = documents.find_document(id) {
        doc.request_animation_frame(AnimationFrameCallback::DevtoolsFramerateTick { actor_name });
    }
}

pub(crate) fn handle_get_css_database(reply: GenericSender<HashMap<String, CssDatabaseProperty>>) {
    let database: HashMap<_, _> = ENABLED_LONGHAND_PROPERTIES
        .iter()
        .map(|l| {
            (
                l.name().into(),
                CssDatabaseProperty {
                    is_inherited: l.inherited(),
                    values: vec![], // TODO: Get allowed values for each property
                    supports: vec![],
                    subproperties: vec![l.name().into()],
                },
            )
        })
        .collect();
    let _ = reply.send(database);
}

pub(crate) fn handle_get_event_listener_info(
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Vec<EventListenerInfo>>,
) {
    let Some(node) = state.find_node_by_unique_id(pipeline, node_id) else {
        reply.send(vec![]).unwrap();
        return;
    };

    let event_listeners = node
        .upcast::<EventTarget>()
        .summarize_event_listeners_for_devtools();
    reply.send(event_listeners).unwrap();
}

pub(crate) fn handle_get_root_node(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Option<NodeInfo>>,
) {
    let info = documents
        .find_document(pipeline)
        .map(DomRoot::upcast::<Node>)
        .inspect(|node| {
            state
                .mut_pipeline_state_for(pipeline)
                .unwrap()
                .register_node(node)
        })
        .map(|document| document.upcast::<Node>().summarize(CanGc::from_cx(cx)));
    reply.send(info).unwrap();
}

pub(crate) fn handle_get_document_element(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Option<NodeInfo>>,
) {
    let info = documents
        .find_document(pipeline)
        .and_then(|document| document.GetDocumentElement())
        .inspect(|element| {
            state
                .mut_pipeline_state_for(pipeline)
                .unwrap()
                .register_node(element.upcast())
        })
        .map(|element| element.upcast::<Node>().summarize(CanGc::from_cx(cx)));
    reply.send(info).unwrap();
}

pub(crate) fn handle_get_children(
    cx: &mut JSContext,
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<NodeInfo>>>,
) {
    let Some(parent) = state.find_node_by_unique_id(pipeline, node_id) else {
        reply.send(None).unwrap();
        return;
    };
    let is_whitespace = |node: &NodeInfo| {
        node.node_type == NodeConstants::TEXT_NODE &&
            node.node_value.as_ref().is_none_or(|v| v.trim().is_empty())
    };
    let mut pipeline_state = state.mut_pipeline_state_for(pipeline).unwrap();

    let inline: Vec<_> = parent
        .children()
        .map(|child| {
            let window = child.owner_window();
            let Some(elem) = child.downcast::<Element>() else {
                return false;
            };
            let computed_style = window.GetComputedStyle(elem, None);
            let display = computed_style.Display();
            display == "inline"
        })
        .collect();

    let mut children = vec![];
    if let Some(shadow_root) = parent.downcast::<Element>().and_then(Element::shadow_root) {
        if !shadow_root.is_user_agent_widget() || pref!(inspector_show_servo_internal_shadow_roots)
        {
            children.push(shadow_root.upcast::<Node>().summarize(CanGc::from_cx(cx)));
        }
    }
    let children_iter = parent.children().enumerate().filter_map(|(i, child)| {
        // Filter whitespace only text nodes that are not inline level
        // https://firefox-source-docs.mozilla.org/devtools-user/page_inspector/how_to/examine_and_edit_html/index.html#whitespace-only-text-nodes
        let prev_inline = i > 0 && inline[i - 1];
        let next_inline = i < inline.len() - 1 && inline[i + 1];
        let is_inline_level = prev_inline && next_inline;

        let info = child.summarize(CanGc::from_cx(cx));
        if is_whitespace(&info) && !is_inline_level {
            return None;
        }
        pipeline_state.register_node(&child);

        Some(info)
    });
    children.extend(children_iter);

    reply.send(Some(children)).unwrap();
}

pub(crate) fn handle_get_attribute_style(
    cx: &mut JSContext,
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<NodeStyle>>>,
) {
    let node = match state.find_node_by_unique_id(pipeline, node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node,
    };

    let Some(elem) = node.downcast::<HTMLElement>() else {
        // the style attribute only works on html elements
        reply.send(None).unwrap();
        return;
    };
    let style = elem.Style(CanGc::from_cx(cx));

    let msg = (0..style.Length())
        .map(|i| {
            let name = style.Item(i);
            NodeStyle {
                name: name.to_string(),
                value: style.GetPropertyValue(name.clone()).to_string(),
                priority: style.GetPropertyPriority(name).to_string(),
            }
        })
        .collect();

    reply.send(Some(msg)).unwrap();
}

fn build_rule_map(
    cx: &mut JSContext,
    list: &crate::dom::css::cssrulelist::CSSRuleList,
    stylesheet_index: usize,
    ancestors: &[AncestorData],
    map: &mut HashMap<usize, MatchedRule>,
) {
    let can_gc = CanGc::from_cx(cx);
    for i in 0..list.Length() {
        let Some(rule) = list.Item(i, can_gc) else {
            continue;
        };

        if let Some(style_rule) = rule.downcast::<CSSStyleRule>() {
            let block_id = style_rule.block_id();
            map.entry(block_id).or_insert_with(|| MatchedRule {
                selector: style_rule.SelectorText().into(),
                stylesheet_index,
                block_id,
                ancestor_data: ancestors.to_vec(),
            });
            continue;
        }

        if let Some(layer_rule) = rule.downcast::<CSSLayerBlockRule>() {
            let name = layer_rule.Name().to_string();
            let mut next = ancestors.to_vec();
            next.push(AncestorData::Layer {
                actor_id: None,
                value: (!name.is_empty()).then_some(name),
            });
            let inner = layer_rule.upcast::<CSSGroupingRule>().CssRules(cx);
            build_rule_map(cx, &inner, stylesheet_index, &next, map);
            continue;
        }

        if let Some(group_rule) = rule.downcast::<CSSGroupingRule>() {
            let inner = group_rule.CssRules(cx);
            build_rule_map(cx, &inner, stylesheet_index, ancestors, map);
        }
    }
}

fn find_rule_by_block_id(
    cx: &mut JSContext,
    list: &crate::dom::css::cssrulelist::CSSRuleList,
    target_block_id: usize,
) -> Option<DomRoot<CSSStyleRule>> {
    let can_gc = CanGc::from_cx(cx);
    for i in 0..list.Length() {
        let Some(rule) = list.Item(i, can_gc) else {
            continue;
        };

        if let Some(style_rule) = rule.downcast::<CSSStyleRule>() {
            if style_rule.block_id() == target_block_id {
                return Some(DomRoot::from_ref(style_rule));
            }
            continue;
        }

        if let Some(group_rule) = rule.downcast::<CSSGroupingRule>() {
            let inner = group_rule.CssRules(cx);
            if let Some(found) = find_rule_by_block_id(cx, &inner, target_block_id) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
pub(crate) fn handle_get_selectors(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<MatchedRule>>>,
) {
    let msg = (|| {
        let node = state.find_node_by_unique_id(pipeline, node_id)?;
        let elem = node.downcast::<Element>()?;
        let document = documents.find_document(pipeline)?;
        let _realm = enter_realm(document.window());
        let owner = node.stylesheet_list_owner();

        let mut decl_map = HashMap::new();
        for i in 0..owner.stylesheet_count() {
            let Some(stylesheet) = owner.stylesheet_at(i) else {
                continue;
            };
            let Ok(list) = stylesheet.GetCssRules(CanGc::from_cx(cx)) else {
                continue;
            };
            build_rule_map(cx, &list, i, &[], &mut decl_map);
        }

        let mut rules = Vec::new();
        let computed = elem.style()?;

        if let Some(rule_node) = computed.rules.as_ref() {
            for rn in rule_node.self_and_ancestors() {
                if let Some(source) = rn.style_source() {
                    let ptr = source.get().raw_ptr().as_ptr() as usize;

                    if let Some(matched) = decl_map.get(&ptr) {
                        rules.push(matched.clone());
                    }
                }
            }
        }

        Some(rules)
    })();

    reply.send(msg).unwrap();
}

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_get_stylesheet_style(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    matched_rule: MatchedRule,
    reply: GenericSender<Option<Vec<NodeStyle>>>,
) {
    let msg = (|| {
        let node = state.find_node_by_unique_id(pipeline, node_id)?;
        let document = documents.find_document(pipeline)?;
        let _realm = enter_realm(document.window());
        let owner = node.stylesheet_list_owner();

        let stylesheet = owner.stylesheet_at(matched_rule.stylesheet_index)?;
        let list = stylesheet.GetCssRules(CanGc::from_cx(cx)).ok()?;

        let style_rule = find_rule_by_block_id(cx, &list, matched_rule.block_id)?;
        let declaration = style_rule.Style(cx);

        Some(
            (0..declaration.Length())
                .map(|i| {
                    let name = declaration.Item(i);
                    NodeStyle {
                        name: name.to_string(),
                        value: declaration.GetPropertyValue(name.clone()).to_string(),
                        priority: declaration.GetPropertyPriority(name).to_string(),
                    }
                })
                .collect(),
        )
    })();

    reply.send(msg).unwrap();
}

pub(crate) fn handle_get_computed_style(
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<NodeStyle>>>,
) {
    let node = match state.find_node_by_unique_id(pipeline, node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node,
    };

    let window = node.owner_window();
    let elem = node
        .downcast::<Element>()
        .expect("This should be an element");
    let computed_style = window.GetComputedStyle(elem, None);

    let msg = (0..computed_style.Length())
        .map(|i| {
            let name = computed_style.Item(i);
            NodeStyle {
                name: name.to_string(),
                value: computed_style.GetPropertyValue(name.clone()).to_string(),
                priority: computed_style.GetPropertyPriority(name).to_string(),
            }
        })
        .collect();

    reply.send(Some(msg)).unwrap();
}

pub(crate) fn handle_get_layout(
    cx: &mut JSContext,
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<(ComputedNodeLayout, AutoMargins)>>,
) {
    let node = match state.find_node_by_unique_id(pipeline, node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node,
    };

    let element = node
        .downcast::<Element>()
        .expect("should be getting layout of element");

    let rect = element.GetBoundingClientRect(CanGc::from_cx(cx));
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;

    let window = node.owner_window();
    let computed_style = window.GetComputedStyle(element, None);
    let computed_layout = ComputedNodeLayout {
        display: computed_style.Display().into(),
        position: computed_style.Position().into(),
        z_index: computed_style.ZIndex().into(),
        box_sizing: computed_style.BoxSizing().into(),
        margin_top: computed_style.MarginTop().into(),
        margin_right: computed_style.MarginRight().into(),
        margin_bottom: computed_style.MarginBottom().into(),
        margin_left: computed_style.MarginLeft().into(),
        border_top_width: computed_style.BorderTopWidth().into(),
        border_right_width: computed_style.BorderRightWidth().into(),
        border_bottom_width: computed_style.BorderBottomWidth().into(),
        border_left_width: computed_style.BorderLeftWidth().into(),
        padding_top: computed_style.PaddingTop().into(),
        padding_right: computed_style.PaddingRight().into(),
        padding_bottom: computed_style.PaddingBottom().into(),
        padding_left: computed_style.PaddingLeft().into(),
        width,
        height,
    };

    let auto_margins = element.determine_auto_margins();
    reply.send(Some((computed_layout, auto_margins))).unwrap();
}

pub(crate) fn handle_get_xpath(
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<String>,
) {
    let Some(node) = state.find_node_by_unique_id(pipeline, node_id) else {
        return reply.send(Default::default()).unwrap();
    };

    let selector = node
        .inclusive_ancestors(ShadowIncluding::Yes)
        .filter_map(|ancestor| {
            let Some(element) = ancestor.downcast::<Element>() else {
                // TODO: figure out how to handle shadow roots here
                return None;
            };

            let mut result = "/".to_owned();
            if *element.namespace() != ns!(html) {
                result.push_str(element.namespace());
                result.push(':');
            }

            result.push_str(element.local_name());

            let would_node_also_match_selector = |sibling: &Node| {
                let Some(sibling) = sibling.downcast::<Element>() else {
                    return false;
                };
                sibling.namespace() == element.namespace() &&
                    sibling.local_name() == element.local_name()
            };

            let matching_elements_before = ancestor
                .preceding_siblings()
                .filter(|node| would_node_also_match_selector(node))
                .count();
            let matching_elements_after = ancestor
                .following_siblings()
                .filter(|node| would_node_also_match_selector(node))
                .count();

            if matching_elements_before + matching_elements_after != 0 {
                // Need to add an index (note that XPath uses 1-based indexing)
                result.push_str(&format!("[{}]", matching_elements_before + 1));
            }

            Some(result)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("");

    reply.send(selector).unwrap();
}

pub(crate) fn handle_modify_attribute(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    modifications: Vec<AttrModification>,
) {
    let Some(document) = documents.find_document(pipeline) else {
        return warn!("document for pipeline id {} is not found", &pipeline);
    };
    let _realm = enter_realm(document.window());

    let node = match state.find_node_by_unique_id(pipeline, node_id) {
        None => {
            return warn!(
                "node id {} for pipeline id {} is not found",
                &node_id, &pipeline
            );
        },
        Some(found_node) => found_node,
    };

    let elem = node
        .downcast::<Element>()
        .expect("should be getting layout of element");

    for modification in modifications {
        match modification.new_value {
            Some(string) => {
                elem.set_attribute(
                    &LocalName::from(modification.attribute_name),
                    AttrValue::String(string),
                    CanGc::from_cx(cx),
                );
            },
            None => elem.RemoveAttribute(
                DOMString::from(modification.attribute_name),
                CanGc::from_cx(cx),
            ),
        }
    }
}

pub(crate) fn handle_modify_rule(
    cx: &mut JSContext,
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    modifications: Vec<RuleModification>,
) {
    let Some(document) = documents.find_document(pipeline) else {
        return warn!("Document for pipeline id {} is not found", &pipeline);
    };
    let _realm = enter_realm(document.window());

    let Some(node) = state.find_node_by_unique_id(pipeline, node_id) else {
        return warn!(
            "Node id {} for pipeline id {} is not found",
            &node_id, &pipeline
        );
    };

    let elem = node
        .downcast::<HTMLElement>()
        .expect("This should be an HTMLElement");
    let style = elem.Style(CanGc::from_cx(cx));

    for modification in modifications {
        let _ = style.SetProperty(
            cx,
            modification.name.into(),
            modification.value.into(),
            modification.priority.into(),
        );
    }
}

pub(crate) fn handle_highlight_dom_node(
    state: &DevtoolsState,
    documents: &DocumentCollection,
    id: PipelineId,
    node_id: Option<&str>,
) {
    let node = node_id.and_then(|node_id| {
        let node = state.find_node_by_unique_id(id, node_id);
        if node.is_none() {
            log::warn!("Node id {node_id} for pipeline id {id} is not found",);
        }
        node
    });

    if let Some(window) = documents.find_window(id) {
        window.Document().highlight_dom_node(node.as_deref());
    }
}

impl Element {
    fn determine_auto_margins(&self) -> AutoMargins {
        let Some(style) = self.style() else {
            return AutoMargins::default();
        };
        let margin = style.get_margin();
        AutoMargins {
            top: margin.margin_top.is_auto(),
            right: margin.margin_right.is_auto(),
            bottom: margin.margin_bottom.is_auto(),
            left: margin.margin_left.is_auto(),
        }
    }
}
