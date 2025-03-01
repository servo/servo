/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use html5ever::interface::QualName;
use html5ever::{local_name, namespace_url, ns, LocalName};
use js::conversions::ToJSValConvertible;
use js::glue::{UnwrapObjectDynamic, UnwrapObjectStatic};
use js::jsapi::{CallArgs, CurrentGlobalOrNull, JSAutoRealm, JSObject};
use js::rust::wrappers::{JS_SetPrototype, JS_WrapObject};
use js::rust::{HandleObject, MutableHandleObject, MutableHandleValue};

use super::utils::ProtoOrIfaceArray;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::{
    HTMLAnchorElementBinding, HTMLAreaElementBinding, HTMLAudioElementBinding,
    HTMLBRElementBinding, HTMLBaseElementBinding, HTMLBodyElementBinding, HTMLButtonElementBinding,
    HTMLCanvasElementBinding, HTMLDListElementBinding, HTMLDataElementBinding,
    HTMLDataListElementBinding, HTMLDetailsElementBinding, HTMLDialogElementBinding,
    HTMLDirectoryElementBinding, HTMLDivElementBinding, HTMLElementBinding,
    HTMLEmbedElementBinding, HTMLFieldSetElementBinding, HTMLFontElementBinding,
    HTMLFormElementBinding, HTMLFrameElementBinding, HTMLFrameSetElementBinding,
    HTMLHRElementBinding, HTMLHeadElementBinding, HTMLHeadingElementBinding,
    HTMLHtmlElementBinding, HTMLIFrameElementBinding, HTMLImageElementBinding,
    HTMLInputElementBinding, HTMLLIElementBinding, HTMLLabelElementBinding,
    HTMLLegendElementBinding, HTMLLinkElementBinding, HTMLMapElementBinding,
    HTMLMenuElementBinding, HTMLMetaElementBinding, HTMLMeterElementBinding, HTMLModElementBinding,
    HTMLOListElementBinding, HTMLObjectElementBinding, HTMLOptGroupElementBinding,
    HTMLOptionElementBinding, HTMLOutputElementBinding, HTMLParagraphElementBinding,
    HTMLParamElementBinding, HTMLPictureElementBinding, HTMLPreElementBinding,
    HTMLProgressElementBinding, HTMLQuoteElementBinding, HTMLScriptElementBinding,
    HTMLSelectElementBinding, HTMLSourceElementBinding, HTMLSpanElementBinding,
    HTMLStyleElementBinding, HTMLTableCaptionElementBinding, HTMLTableCellElementBinding,
    HTMLTableColElementBinding, HTMLTableElementBinding, HTMLTableRowElementBinding,
    HTMLTableSectionElementBinding, HTMLTemplateElementBinding, HTMLTextAreaElementBinding,
    HTMLTimeElementBinding, HTMLTitleElementBinding, HTMLTrackElementBinding,
    HTMLUListElementBinding, HTMLVideoElementBinding,
};
use crate::dom::bindings::codegen::PrototypeList;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{throw_constructor_without_new, throw_dom_exception, Error};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::interface::get_desired_proto;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::create::create_native_html_element;
use crate::dom::customelementregistry::{ConstructionStackEntry, CustomElementState};
use crate::dom::element::{Element, ElementCreator};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext, JSContext as SafeJSContext};
use crate::script_thread::ScriptThread;

// https://html.spec.whatwg.org/multipage/#htmlconstructor
unsafe fn html_constructor(
    cx: JSContext,
    global: &GlobalScope,
    call_args: &CallArgs,
    check_type: fn(&Element) -> bool,
    proto_id: PrototypeList::ID,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
    can_gc: CanGc,
) -> Result<(), ()> {
    let window = global.downcast::<Window>().unwrap();
    let document = window.Document();

    // Step 1
    let registry = window.CustomElements();

    // Step 2 https://html.spec.whatwg.org/multipage/#htmlconstructor
    // The custom element definition cannot use an element interface as its constructor

    // The new_target might be a cross-compartment wrapper. Get the underlying object
    // so we can do the spec's object-identity checks.
    rooted!(in(*cx) let new_target_unwrapped = UnwrapObjectDynamic(call_args.new_target().to_object(), *cx, true));
    if new_target_unwrapped.is_null() {
        throw_dom_exception(
            cx,
            global,
            Error::Type("new.target is null".to_owned()),
            can_gc,
        );
        return Err(());
    }
    if call_args.callee() == new_target_unwrapped.get() {
        throw_dom_exception(
            cx,
            global,
            Error::Type("new.target must not be the active function object".to_owned()),
            can_gc,
        );
        return Err(());
    }

    // Step 3
    rooted!(in(*cx) let new_target = call_args.new_target().to_object());
    let definition = match registry.lookup_definition_by_constructor(new_target.handle()) {
        Some(definition) => definition,
        None => {
            throw_dom_exception(
                cx,
                global,
                Error::Type("No custom element definition found for new.target".to_owned()),
                can_gc,
            );
            return Err(());
        },
    };

    rooted!(in(*cx) let callee = UnwrapObjectStatic(call_args.callee()));
    if callee.is_null() {
        throw_dom_exception(cx, global, Error::Security, can_gc);
        return Err(());
    }

    {
        let _ac = JSAutoRealm::new(*cx, callee.get());
        rooted!(in(*cx) let mut constructor = ptr::null_mut::<JSObject>());
        rooted!(in(*cx) let global_object = CurrentGlobalOrNull(*cx));

        if definition.is_autonomous() {
            // Step 4
            // Since this element is autonomous, its active function object must be the HTMLElement

            // Retrieve the constructor object for HTMLElement
            HTMLElementBinding::GetConstructorObject(
                cx,
                global_object.handle(),
                constructor.handle_mut(),
            );
        } else {
            // Step 5
            get_constructor_object_from_local_name(
                definition.local_name.clone(),
                cx,
                global_object.handle(),
                constructor.handle_mut(),
            );
        }
        // Callee must be the same as the element interface's constructor object.
        if constructor.get() != callee.get() {
            throw_dom_exception(
                cx,
                global,
                Error::Type("Custom element does not extend the proper interface".to_owned()),
                can_gc,
            );
            return Err(());
        }
    }

    // Step 6
    rooted!(in(*cx) let mut prototype = ptr::null_mut::<JSObject>());
    get_desired_proto(cx, call_args, proto_id, creator, prototype.handle_mut())?;

    let entry = definition.construction_stack.borrow().last().cloned();
    let result = match entry {
        // Step 8
        None => {
            // Step 8.1
            let name = QualName::new(None, ns!(html), definition.local_name.clone());
            // Any prototype used to create these elements will be overwritten before returning
            // from this function, so we don't bother overwriting the defaults here.
            let element = if definition.is_autonomous() {
                DomRoot::upcast(HTMLElement::new(name.local, None, &document, None, can_gc))
            } else {
                create_native_html_element(
                    name,
                    None,
                    &document,
                    ElementCreator::ScriptCreated,
                    None,
                )
            };

            // Step 8.2 is performed in the generated caller code.

            // Step 8.3
            element.set_custom_element_state(CustomElementState::Custom);

            // Step 8.4
            element.set_custom_element_definition(definition.clone());

            // Step 8.5
            if !check_type(&element) {
                throw_dom_exception(cx, global, Error::InvalidState, can_gc);
                return Err(());
            } else {
                element
            }
        },
        // Step 9
        Some(ConstructionStackEntry::Element(element)) => {
            // Step 11 is performed in the generated caller code.

            // Step 12
            let mut construction_stack = definition.construction_stack.borrow_mut();
            construction_stack.pop();
            construction_stack.push(ConstructionStackEntry::AlreadyConstructedMarker);

            // Step 13
            if !check_type(&element) {
                throw_dom_exception(cx, global, Error::InvalidState, can_gc);
                return Err(());
            } else {
                element
            }
        },
        // Step 10
        Some(ConstructionStackEntry::AlreadyConstructedMarker) => {
            let s = "Top of construction stack marked AlreadyConstructed due to \
                     a custom element constructor constructing itself after super()"
                .to_string();
            throw_dom_exception(cx, global, Error::Type(s), can_gc);
            return Err(());
        },
    };

    rooted!(in(*cx) let mut element = result.reflector().get_jsobject().get());
    if !JS_WrapObject(*cx, element.handle_mut()) {
        return Err(());
    }

    JS_SetPrototype(*cx, element.handle(), prototype.handle());

    result.to_jsval(*cx, MutableHandleValue::from_raw(call_args.rval()));
    Ok(())
}

/// Returns the constructor object for the element associated with the
/// given local name. This list should only include elements marked with the
/// [HTMLConstructor](https://html.spec.whatwg.org/multipage/#htmlconstructor)
/// extended attribute.
fn get_constructor_object_from_local_name(
    name: LocalName,
    cx: JSContext,
    global: HandleObject,
    rval: MutableHandleObject,
) -> bool {
    let constructor_fn = match name {
        local_name!("a") => HTMLAnchorElementBinding::GetConstructorObject,
        local_name!("abbr") => HTMLElementBinding::GetConstructorObject,
        local_name!("acronym") => HTMLElementBinding::GetConstructorObject,
        local_name!("address") => HTMLElementBinding::GetConstructorObject,
        local_name!("area") => HTMLAreaElementBinding::GetConstructorObject,
        local_name!("article") => HTMLElementBinding::GetConstructorObject,
        local_name!("aside") => HTMLElementBinding::GetConstructorObject,
        local_name!("audio") => HTMLAudioElementBinding::GetConstructorObject,
        local_name!("b") => HTMLElementBinding::GetConstructorObject,
        local_name!("base") => HTMLBaseElementBinding::GetConstructorObject,
        local_name!("bdi") => HTMLElementBinding::GetConstructorObject,
        local_name!("bdo") => HTMLElementBinding::GetConstructorObject,
        local_name!("big") => HTMLElementBinding::GetConstructorObject,
        local_name!("blockquote") => HTMLQuoteElementBinding::GetConstructorObject,
        local_name!("body") => HTMLBodyElementBinding::GetConstructorObject,
        local_name!("br") => HTMLBRElementBinding::GetConstructorObject,
        local_name!("button") => HTMLButtonElementBinding::GetConstructorObject,
        local_name!("canvas") => HTMLCanvasElementBinding::GetConstructorObject,
        local_name!("caption") => HTMLTableCaptionElementBinding::GetConstructorObject,
        local_name!("center") => HTMLElementBinding::GetConstructorObject,
        local_name!("cite") => HTMLElementBinding::GetConstructorObject,
        local_name!("code") => HTMLElementBinding::GetConstructorObject,
        local_name!("col") => HTMLTableColElementBinding::GetConstructorObject,
        local_name!("colgroup") => HTMLTableColElementBinding::GetConstructorObject,
        local_name!("data") => HTMLDataElementBinding::GetConstructorObject,
        local_name!("datalist") => HTMLDataListElementBinding::GetConstructorObject,
        local_name!("dd") => HTMLElementBinding::GetConstructorObject,
        local_name!("del") => HTMLModElementBinding::GetConstructorObject,
        local_name!("details") => HTMLDetailsElementBinding::GetConstructorObject,
        local_name!("dfn") => HTMLElementBinding::GetConstructorObject,
        local_name!("dialog") => HTMLDialogElementBinding::GetConstructorObject,
        local_name!("dir") => HTMLDirectoryElementBinding::GetConstructorObject,
        local_name!("div") => HTMLDivElementBinding::GetConstructorObject,
        local_name!("dl") => HTMLDListElementBinding::GetConstructorObject,
        local_name!("dt") => HTMLElementBinding::GetConstructorObject,
        local_name!("em") => HTMLElementBinding::GetConstructorObject,
        local_name!("embed") => HTMLEmbedElementBinding::GetConstructorObject,
        local_name!("fieldset") => HTMLFieldSetElementBinding::GetConstructorObject,
        local_name!("figcaption") => HTMLElementBinding::GetConstructorObject,
        local_name!("figure") => HTMLElementBinding::GetConstructorObject,
        local_name!("font") => HTMLFontElementBinding::GetConstructorObject,
        local_name!("footer") => HTMLElementBinding::GetConstructorObject,
        local_name!("form") => HTMLFormElementBinding::GetConstructorObject,
        local_name!("frame") => HTMLFrameElementBinding::GetConstructorObject,
        local_name!("frameset") => HTMLFrameSetElementBinding::GetConstructorObject,
        local_name!("h1") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("h2") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("h3") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("h4") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("h5") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("h6") => HTMLHeadingElementBinding::GetConstructorObject,
        local_name!("head") => HTMLHeadElementBinding::GetConstructorObject,
        local_name!("header") => HTMLElementBinding::GetConstructorObject,
        local_name!("hgroup") => HTMLElementBinding::GetConstructorObject,
        local_name!("hr") => HTMLHRElementBinding::GetConstructorObject,
        local_name!("html") => HTMLHtmlElementBinding::GetConstructorObject,
        local_name!("i") => HTMLElementBinding::GetConstructorObject,
        local_name!("iframe") => HTMLIFrameElementBinding::GetConstructorObject,
        local_name!("img") => HTMLImageElementBinding::GetConstructorObject,
        local_name!("input") => HTMLInputElementBinding::GetConstructorObject,
        local_name!("ins") => HTMLModElementBinding::GetConstructorObject,
        local_name!("kbd") => HTMLElementBinding::GetConstructorObject,
        local_name!("label") => HTMLLabelElementBinding::GetConstructorObject,
        local_name!("legend") => HTMLLegendElementBinding::GetConstructorObject,
        local_name!("li") => HTMLLIElementBinding::GetConstructorObject,
        local_name!("link") => HTMLLinkElementBinding::GetConstructorObject,
        local_name!("listing") => HTMLPreElementBinding::GetConstructorObject,
        local_name!("main") => HTMLElementBinding::GetConstructorObject,
        local_name!("map") => HTMLMapElementBinding::GetConstructorObject,
        local_name!("mark") => HTMLElementBinding::GetConstructorObject,
        local_name!("marquee") => HTMLElementBinding::GetConstructorObject,
        local_name!("menu") => HTMLMenuElementBinding::GetConstructorObject,
        local_name!("meta") => HTMLMetaElementBinding::GetConstructorObject,
        local_name!("meter") => HTMLMeterElementBinding::GetConstructorObject,
        local_name!("nav") => HTMLElementBinding::GetConstructorObject,
        local_name!("nobr") => HTMLElementBinding::GetConstructorObject,
        local_name!("noframes") => HTMLElementBinding::GetConstructorObject,
        local_name!("noscript") => HTMLElementBinding::GetConstructorObject,
        local_name!("object") => HTMLObjectElementBinding::GetConstructorObject,
        local_name!("ol") => HTMLOListElementBinding::GetConstructorObject,
        local_name!("optgroup") => HTMLOptGroupElementBinding::GetConstructorObject,
        local_name!("option") => HTMLOptionElementBinding::GetConstructorObject,
        local_name!("output") => HTMLOutputElementBinding::GetConstructorObject,
        local_name!("p") => HTMLParagraphElementBinding::GetConstructorObject,
        local_name!("param") => HTMLParamElementBinding::GetConstructorObject,
        local_name!("picture") => HTMLPictureElementBinding::GetConstructorObject,
        local_name!("plaintext") => HTMLPreElementBinding::GetConstructorObject,
        local_name!("pre") => HTMLPreElementBinding::GetConstructorObject,
        local_name!("progress") => HTMLProgressElementBinding::GetConstructorObject,
        local_name!("q") => HTMLQuoteElementBinding::GetConstructorObject,
        local_name!("rp") => HTMLElementBinding::GetConstructorObject,
        local_name!("rt") => HTMLElementBinding::GetConstructorObject,
        local_name!("ruby") => HTMLElementBinding::GetConstructorObject,
        local_name!("s") => HTMLElementBinding::GetConstructorObject,
        local_name!("samp") => HTMLElementBinding::GetConstructorObject,
        local_name!("script") => HTMLScriptElementBinding::GetConstructorObject,
        local_name!("section") => HTMLElementBinding::GetConstructorObject,
        local_name!("select") => HTMLSelectElementBinding::GetConstructorObject,
        local_name!("small") => HTMLElementBinding::GetConstructorObject,
        local_name!("source") => HTMLSourceElementBinding::GetConstructorObject,
        local_name!("span") => HTMLSpanElementBinding::GetConstructorObject,
        local_name!("strike") => HTMLElementBinding::GetConstructorObject,
        local_name!("strong") => HTMLElementBinding::GetConstructorObject,
        local_name!("style") => HTMLStyleElementBinding::GetConstructorObject,
        local_name!("sub") => HTMLElementBinding::GetConstructorObject,
        local_name!("summary") => HTMLElementBinding::GetConstructorObject,
        local_name!("sup") => HTMLElementBinding::GetConstructorObject,
        local_name!("table") => HTMLTableElementBinding::GetConstructorObject,
        local_name!("tbody") => HTMLTableSectionElementBinding::GetConstructorObject,
        local_name!("td") => HTMLTableCellElementBinding::GetConstructorObject,
        local_name!("template") => HTMLTemplateElementBinding::GetConstructorObject,
        local_name!("textarea") => HTMLTextAreaElementBinding::GetConstructorObject,
        local_name!("tfoot") => HTMLTableSectionElementBinding::GetConstructorObject,
        local_name!("th") => HTMLTableCellElementBinding::GetConstructorObject,
        local_name!("thead") => HTMLTableSectionElementBinding::GetConstructorObject,
        local_name!("time") => HTMLTimeElementBinding::GetConstructorObject,
        local_name!("title") => HTMLTitleElementBinding::GetConstructorObject,
        local_name!("tr") => HTMLTableRowElementBinding::GetConstructorObject,
        local_name!("tt") => HTMLElementBinding::GetConstructorObject,
        local_name!("track") => HTMLTrackElementBinding::GetConstructorObject,
        local_name!("u") => HTMLElementBinding::GetConstructorObject,
        local_name!("ul") => HTMLUListElementBinding::GetConstructorObject,
        local_name!("var") => HTMLElementBinding::GetConstructorObject,
        local_name!("video") => HTMLVideoElementBinding::GetConstructorObject,
        local_name!("wbr") => HTMLElementBinding::GetConstructorObject,
        local_name!("xmp") => HTMLPreElementBinding::GetConstructorObject,
        _ => return false,
    };
    constructor_fn(cx, global, rval);
    true
}

pub(crate) fn pop_current_element_queue(can_gc: CanGc) {
    ScriptThread::pop_current_element_queue(can_gc);
}

pub(crate) fn push_new_element_queue() {
    ScriptThread::push_new_element_queue();
}

pub(crate) unsafe fn call_html_constructor<T: DerivedFrom<Element> + DomObject>(
    cx: JSContext,
    args: &CallArgs,
    global: &GlobalScope,
    proto_id: PrototypeList::ID,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
    can_gc: CanGc,
) -> bool {
    fn element_derives_interface<T: DerivedFrom<Element>>(element: &Element) -> bool {
        element.is::<T>()
    }

    html_constructor(
        cx,
        global,
        args,
        element_derives_interface::<T>,
        proto_id,
        creator,
        can_gc,
    )
    .is_ok()
}

pub(crate) unsafe fn call_default_constructor<D: crate::DomTypes>(
    cx: JSContext,
    args: &CallArgs,
    global: &D::GlobalScope,
    proto_id: PrototypeList::ID,
    ctor_name: &str,
    creator: unsafe fn(JSContext, HandleObject, *mut ProtoOrIfaceArray),
    constructor: impl FnOnce(JSContext, &CallArgs, &D::GlobalScope, HandleObject) -> bool,
) -> bool {
    if !args.is_constructing() {
        throw_constructor_without_new(cx, ctor_name);
        return false;
    }

    rooted!(in(*cx) let mut desired_proto = ptr::null_mut::<JSObject>());
    let proto_result = get_desired_proto(cx, args, proto_id, creator, desired_proto.handle_mut());
    if proto_result.is_err() {
        return false;
    }

    constructor(cx, args, global, desired_proto.handle())
}
