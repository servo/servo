/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::str;

use base::generic_channel::GenericSender;
use base::id::PipelineId;
use devtools_traits::{
    AttrModification, AutoMargins, ComputedNodeLayout, CssDatabaseProperty, EvaluateJSReply,
    EventListenerInfo, NodeInfo, NodeStyle, RuleModification, TimelineMarker, TimelineMarkerType,
};
use js::context::JSContext;
use js::conversions::jsstr_to_string;
use js::jsval::UndefinedValue;
use js::rust::ToString;
use markup5ever::{LocalName, ns};
use rustc_hash::FxHashMap;
use script_bindings::root::Dom;
use servo_config::pref;
use style::attr::AttrValue;
use uuid::Uuid;

use crate::document_collection::DocumentCollection;
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
use crate::dom::bindings::conversions::{ConversionResult, FromJSValConvertible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::NoTrace;
use crate::dom::css::cssstyledeclaration::ENABLED_LONGHAND_PROPERTIES;
use crate::dom::css::cssstylerule::CSSStyleRule;
use crate::dom::document::AnimationFrameCallback;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::types::{EventTarget, HTMLElement};
use crate::realms::{enter_auto_realm, enter_realm};
use crate::script_runtime::{CanGc, IntroductionType};

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

#[expect(unsafe_code)]
pub(crate) fn handle_evaluate_js(
    global: &GlobalScope,
    eval: String,
    reply: GenericSender<EvaluateJSReply>,
    cx: &mut JSContext,
) {
    let result = unsafe {
        let mut realm = enter_auto_realm(cx, global);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut rval = UndefinedValue());
        // TODO: run code with SpiderMonkey Debugger API, like Firefox does
        // <https://searchfox.org/mozilla-central/rev/f6a806c38c459e0e0d797d264ca0e8ad46005105/devtools/server/actors/webconsole/eval-with-debugger.js#270>
        _ = global.evaluate_js_on_global(
            cx,
            eval.into(),
            "<eval>",
            Some(IntroductionType::DEBUGGER_EVAL),
            rval.handle_mut(),
        );

        if rval.is_undefined() {
            EvaluateJSReply::VoidValue
        } else if rval.is_boolean() {
            EvaluateJSReply::BooleanValue(rval.to_boolean())
        } else if rval.is_double() || rval.is_int32() {
            EvaluateJSReply::NumberValue(
                match FromJSValConvertible::from_jsval(cx.raw_cx(), rval.handle(), ()) {
                    Ok(ConversionResult::Success(v)) => v,
                    _ => unreachable!(),
                },
            )
        } else if rval.is_string() {
            let jsstr = std::ptr::NonNull::new(rval.to_string()).unwrap();
            EvaluateJSReply::StringValue(jsstr_to_string(cx.raw_cx(), jsstr))
        } else if rval.is_null() {
            EvaluateJSReply::NullValue
        } else {
            assert!(rval.is_object());

            let jsstr = std::ptr::NonNull::new(ToString(cx.raw_cx(), rval.handle())).unwrap();
            let class_name = jsstr_to_string(cx.raw_cx(), jsstr);

            EvaluateJSReply::ActorValue {
                class: class_name,
                uuid: Uuid::new_v4().to_string(),
            }
        }
    };
    reply.send(result).unwrap();
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
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Option<NodeInfo>>,
    can_gc: CanGc,
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
        .map(|document| document.upcast::<Node>().summarize(can_gc));
    reply.send(info).unwrap();
}

pub(crate) fn handle_get_document_element(
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    reply: GenericSender<Option<NodeInfo>>,
    can_gc: CanGc,
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
        .map(|element| element.upcast::<Node>().summarize(can_gc));
    reply.send(info).unwrap();
}

pub(crate) fn handle_get_children(
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<NodeInfo>>>,
    can_gc: CanGc,
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
            children.push(shadow_root.upcast::<Node>().summarize(can_gc));
        }
    }
    let children_iter = parent.children().enumerate().filter_map(|(i, child)| {
        // Filter whitespace only text nodes that are not inline level
        // https://firefox-source-docs.mozilla.org/devtools-user/page_inspector/how_to/examine_and_edit_html/index.html#whitespace-only-text-nodes
        let prev_inline = i > 0 && inline[i - 1];
        let next_inline = i < inline.len() - 1 && inline[i + 1];
        let is_inline_level = prev_inline && next_inline;

        let info = child.summarize(can_gc);
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
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<NodeStyle>>>,
    can_gc: CanGc,
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
    let style = elem.Style(can_gc);

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

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_get_stylesheet_style(
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    selector: String,
    stylesheet: usize,
    reply: GenericSender<Option<Vec<NodeStyle>>>,
    can_gc: CanGc,
) {
    let msg = (|| {
        let node = state.find_node_by_unique_id(pipeline, node_id)?;

        let document = documents.find_document(pipeline)?;
        let _realm = enter_realm(document.window());
        let owner = node.stylesheet_list_owner();

        let stylesheet = owner.stylesheet_at(stylesheet)?;
        let list = stylesheet.GetCssRules(can_gc).ok()?;

        let styles = (0..list.Length())
            .filter_map(move |i| {
                let rule = list.Item(i, can_gc)?;
                let style = rule.downcast::<CSSStyleRule>()?;
                if selector != style.SelectorText() {
                    return None;
                };
                Some(style.Style(can_gc))
            })
            .flat_map(|style| {
                (0..style.Length()).map(move |i| {
                    let name = style.Item(i);
                    NodeStyle {
                        name: name.to_string(),
                        value: style.GetPropertyValue(name.clone()).to_string(),
                        priority: style.GetPropertyPriority(name).to_string(),
                    }
                })
            })
            .collect();

        Some(styles)
    })();

    reply.send(msg).unwrap();
}

#[cfg_attr(crown, expect(crown::unrooted_must_root))]
pub(crate) fn handle_get_selectors(
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<Vec<(String, usize)>>>,
    can_gc: CanGc,
) {
    let msg = (|| {
        let node = state.find_node_by_unique_id(pipeline, node_id)?;

        let document = documents.find_document(pipeline)?;
        let _realm = enter_realm(document.window());
        let owner = node.stylesheet_list_owner();

        let rules = (0..owner.stylesheet_count())
            .filter_map(|i| {
                let stylesheet = owner.stylesheet_at(i)?;
                let list = stylesheet.GetCssRules(can_gc).ok()?;
                let elem = node.downcast::<Element>()?;

                Some((0..list.Length()).filter_map(move |j| {
                    let rule = list.Item(j, can_gc)?;
                    let style = rule.downcast::<CSSStyleRule>()?;
                    let selector = style.SelectorText();
                    elem.Matches(selector.clone()).ok()?.then_some(())?;
                    Some((selector.into(), i))
                }))
            })
            .flatten()
            .collect();

        Some(rules)
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
    state: &DevtoolsState,
    pipeline: PipelineId,
    node_id: &str,
    reply: GenericSender<Option<(ComputedNodeLayout, AutoMargins)>>,
    can_gc: CanGc,
) {
    let node = match state.find_node_by_unique_id(pipeline, node_id) {
        None => return reply.send(None).unwrap(),
        Some(found_node) => found_node,
    };
    let auto_margins = determine_auto_margins(&node);

    let elem = node
        .downcast::<Element>()
        .expect("should be getting layout of element");
    let rect = elem.GetBoundingClientRect(can_gc);
    let width = rect.Width() as f32;
    let height = rect.Height() as f32;

    let window = node.owner_window();
    let computed_style = window.GetComputedStyle(elem, None);
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
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    modifications: Vec<AttrModification>,
    can_gc: CanGc,
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
                    can_gc,
                );
            },
            None => elem.RemoveAttribute(DOMString::from(modification.attribute_name), can_gc),
        }
    }
}

pub(crate) fn handle_modify_rule(
    state: &DevtoolsState,
    documents: &DocumentCollection,
    pipeline: PipelineId,
    node_id: &str,
    modifications: Vec<RuleModification>,
    can_gc: CanGc,
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
    let style = elem.Style(can_gc);

    for modification in modifications {
        let _ = style.SetProperty(
            modification.name.into(),
            modification.value.into(),
            modification.priority.into(),
            can_gc,
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

fn determine_auto_margins(node: &Node) -> AutoMargins {
    let Some(style) = node.style() else {
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
