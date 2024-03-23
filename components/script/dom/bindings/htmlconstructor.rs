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
use crate::dom::bindings::error::{throw_dom_exception, Error};
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
use crate::script_runtime::{JSContext, JSContext as SafeJSContext};
use crate::script_thread::ScriptThread;

// https://html.spec.whatwg.org/multipage/#htmlconstructor
unsafe fn html_constructor(
    cx: JSContext,
    window: &Window,
    call_args: &CallArgs,
    check_type: fn(&Element) -> bool,
    proto_id: PrototypeList::ID,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
) -> Result<(), ()> {
    let document = window.Document();
    let global = window.upcast::<GlobalScope>();

    // Step 1
    let registry = window.CustomElements();

    // Step 2 https://html.spec.whatwg.org/multipage/#htmlconstructor
    // The custom element definition cannot use an element interface as its constructor

    // The new_target might be a cross-compartment wrapper. Get the underlying object
    // so we can do the spec's object-identity checks.
    rooted!(in(*cx) let new_target_unwrapped = UnwrapObjectDynamic(call_args.new_target().to_object(), *cx, true));
    if new_target_unwrapped.is_null() {
        throw_dom_exception(cx, global, Error::Type("new.target is null".to_owned()));
        return Err(());
    }
    if call_args.callee() == new_target_unwrapped.get() {
        throw_dom_exception(
            cx,
            global,
            Error::Type("new.target must not be the active function object".to_owned()),
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
            );
            return Err(());
        },
    };

    rooted!(in(*cx) let callee = UnwrapObjectStatic(call_args.callee()));
    if callee.is_null() {
        throw_dom_exception(cx, global, Error::Security);
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
                DomRoot::upcast(HTMLElement::new(name.local, None, &document, None))
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
                throw_dom_exception(cx, global, Error::InvalidState);
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
                throw_dom_exception(cx, global, Error::InvalidState);
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
            throw_dom_exception(cx, global, Error::Type(s));
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
pub fn get_constructor_object_from_local_name(
    name: LocalName,
    cx: JSContext,
    global: HandleObject,
    rval: MutableHandleObject,
) -> bool {
    macro_rules! get_constructor(
        ($binding:ident) => ({
            $binding::GetConstructorObject(cx, global, rval);
            true
        })
    );

    match name {
        local_name!("a") => get_constructor!(HTMLAnchorElementBinding),
        local_name!("abbr") => get_constructor!(HTMLElementBinding),
        local_name!("acronym") => get_constructor!(HTMLElementBinding),
        local_name!("address") => get_constructor!(HTMLElementBinding),
        local_name!("area") => get_constructor!(HTMLAreaElementBinding),
        local_name!("article") => get_constructor!(HTMLElementBinding),
        local_name!("aside") => get_constructor!(HTMLElementBinding),
        local_name!("audio") => get_constructor!(HTMLAudioElementBinding),
        local_name!("b") => get_constructor!(HTMLElementBinding),
        local_name!("base") => get_constructor!(HTMLBaseElementBinding),
        local_name!("bdi") => get_constructor!(HTMLElementBinding),
        local_name!("bdo") => get_constructor!(HTMLElementBinding),
        local_name!("big") => get_constructor!(HTMLElementBinding),
        local_name!("blockquote") => get_constructor!(HTMLQuoteElementBinding),
        local_name!("body") => get_constructor!(HTMLBodyElementBinding),
        local_name!("br") => get_constructor!(HTMLBRElementBinding),
        local_name!("button") => get_constructor!(HTMLButtonElementBinding),
        local_name!("canvas") => get_constructor!(HTMLCanvasElementBinding),
        local_name!("caption") => get_constructor!(HTMLTableCaptionElementBinding),
        local_name!("center") => get_constructor!(HTMLElementBinding),
        local_name!("cite") => get_constructor!(HTMLElementBinding),
        local_name!("code") => get_constructor!(HTMLElementBinding),
        local_name!("col") => get_constructor!(HTMLTableColElementBinding),
        local_name!("colgroup") => get_constructor!(HTMLTableColElementBinding),
        local_name!("data") => get_constructor!(HTMLDataElementBinding),
        local_name!("datalist") => get_constructor!(HTMLDataListElementBinding),
        local_name!("dd") => get_constructor!(HTMLElementBinding),
        local_name!("del") => get_constructor!(HTMLModElementBinding),
        local_name!("details") => get_constructor!(HTMLDetailsElementBinding),
        local_name!("dfn") => get_constructor!(HTMLElementBinding),
        local_name!("dialog") => get_constructor!(HTMLDialogElementBinding),
        local_name!("dir") => get_constructor!(HTMLDirectoryElementBinding),
        local_name!("div") => get_constructor!(HTMLDivElementBinding),
        local_name!("dl") => get_constructor!(HTMLDListElementBinding),
        local_name!("dt") => get_constructor!(HTMLElementBinding),
        local_name!("em") => get_constructor!(HTMLElementBinding),
        local_name!("embed") => get_constructor!(HTMLEmbedElementBinding),
        local_name!("fieldset") => get_constructor!(HTMLFieldSetElementBinding),
        local_name!("figcaption") => get_constructor!(HTMLElementBinding),
        local_name!("figure") => get_constructor!(HTMLElementBinding),
        local_name!("font") => get_constructor!(HTMLFontElementBinding),
        local_name!("footer") => get_constructor!(HTMLElementBinding),
        local_name!("form") => get_constructor!(HTMLFormElementBinding),
        local_name!("frame") => get_constructor!(HTMLFrameElementBinding),
        local_name!("frameset") => get_constructor!(HTMLFrameSetElementBinding),
        local_name!("h1") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h2") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h3") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h4") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h5") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("h6") => get_constructor!(HTMLHeadingElementBinding),
        local_name!("head") => get_constructor!(HTMLHeadElementBinding),
        local_name!("header") => get_constructor!(HTMLElementBinding),
        local_name!("hgroup") => get_constructor!(HTMLElementBinding),
        local_name!("hr") => get_constructor!(HTMLHRElementBinding),
        local_name!("html") => get_constructor!(HTMLHtmlElementBinding),
        local_name!("i") => get_constructor!(HTMLElementBinding),
        local_name!("iframe") => get_constructor!(HTMLIFrameElementBinding),
        local_name!("img") => get_constructor!(HTMLImageElementBinding),
        local_name!("input") => get_constructor!(HTMLInputElementBinding),
        local_name!("ins") => get_constructor!(HTMLModElementBinding),
        local_name!("kbd") => get_constructor!(HTMLElementBinding),
        local_name!("label") => get_constructor!(HTMLLabelElementBinding),
        local_name!("legend") => get_constructor!(HTMLLegendElementBinding),
        local_name!("li") => get_constructor!(HTMLLIElementBinding),
        local_name!("link") => get_constructor!(HTMLLinkElementBinding),
        local_name!("listing") => get_constructor!(HTMLPreElementBinding),
        local_name!("main") => get_constructor!(HTMLElementBinding),
        local_name!("map") => get_constructor!(HTMLMapElementBinding),
        local_name!("mark") => get_constructor!(HTMLElementBinding),
        local_name!("marquee") => get_constructor!(HTMLElementBinding),
        local_name!("menu") => get_constructor!(HTMLMenuElementBinding),
        local_name!("meta") => get_constructor!(HTMLMetaElementBinding),
        local_name!("meter") => get_constructor!(HTMLMeterElementBinding),
        local_name!("nav") => get_constructor!(HTMLElementBinding),
        local_name!("nobr") => get_constructor!(HTMLElementBinding),
        local_name!("noframes") => get_constructor!(HTMLElementBinding),
        local_name!("noscript") => get_constructor!(HTMLElementBinding),
        local_name!("object") => get_constructor!(HTMLObjectElementBinding),
        local_name!("ol") => get_constructor!(HTMLOListElementBinding),
        local_name!("optgroup") => get_constructor!(HTMLOptGroupElementBinding),
        local_name!("option") => get_constructor!(HTMLOptionElementBinding),
        local_name!("output") => get_constructor!(HTMLOutputElementBinding),
        local_name!("p") => get_constructor!(HTMLParagraphElementBinding),
        local_name!("param") => get_constructor!(HTMLParamElementBinding),
        local_name!("picture") => get_constructor!(HTMLPictureElementBinding),
        local_name!("plaintext") => get_constructor!(HTMLPreElementBinding),
        local_name!("pre") => get_constructor!(HTMLPreElementBinding),
        local_name!("progress") => get_constructor!(HTMLProgressElementBinding),
        local_name!("q") => get_constructor!(HTMLQuoteElementBinding),
        local_name!("rp") => get_constructor!(HTMLElementBinding),
        local_name!("rt") => get_constructor!(HTMLElementBinding),
        local_name!("ruby") => get_constructor!(HTMLElementBinding),
        local_name!("s") => get_constructor!(HTMLElementBinding),
        local_name!("samp") => get_constructor!(HTMLElementBinding),
        local_name!("script") => get_constructor!(HTMLScriptElementBinding),
        local_name!("section") => get_constructor!(HTMLElementBinding),
        local_name!("select") => get_constructor!(HTMLSelectElementBinding),
        local_name!("small") => get_constructor!(HTMLElementBinding),
        local_name!("source") => get_constructor!(HTMLSourceElementBinding),
        local_name!("span") => get_constructor!(HTMLSpanElementBinding),
        local_name!("strike") => get_constructor!(HTMLElementBinding),
        local_name!("strong") => get_constructor!(HTMLElementBinding),
        local_name!("style") => get_constructor!(HTMLStyleElementBinding),
        local_name!("sub") => get_constructor!(HTMLElementBinding),
        local_name!("summary") => get_constructor!(HTMLElementBinding),
        local_name!("sup") => get_constructor!(HTMLElementBinding),
        local_name!("table") => get_constructor!(HTMLTableElementBinding),
        local_name!("tbody") => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("td") => get_constructor!(HTMLTableCellElementBinding),
        local_name!("template") => get_constructor!(HTMLTemplateElementBinding),
        local_name!("textarea") => get_constructor!(HTMLTextAreaElementBinding),
        local_name!("tfoot") => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("th") => get_constructor!(HTMLTableCellElementBinding),
        local_name!("thead") => get_constructor!(HTMLTableSectionElementBinding),
        local_name!("time") => get_constructor!(HTMLTimeElementBinding),
        local_name!("title") => get_constructor!(HTMLTitleElementBinding),
        local_name!("tr") => get_constructor!(HTMLTableRowElementBinding),
        local_name!("tt") => get_constructor!(HTMLElementBinding),
        local_name!("track") => get_constructor!(HTMLTrackElementBinding),
        local_name!("u") => get_constructor!(HTMLElementBinding),
        local_name!("ul") => get_constructor!(HTMLUListElementBinding),
        local_name!("var") => get_constructor!(HTMLElementBinding),
        local_name!("video") => get_constructor!(HTMLVideoElementBinding),
        local_name!("wbr") => get_constructor!(HTMLElementBinding),
        local_name!("xmp") => get_constructor!(HTMLPreElementBinding),
        _ => false,
    }
}

pub fn pop_current_element_queue() {
    ScriptThread::pop_current_element_queue();
}

pub fn push_new_element_queue() {
    ScriptThread::push_new_element_queue();
}

pub(crate) unsafe fn call_html_constructor<T: DerivedFrom<Element> + DomObject>(
    cx: JSContext,
    args: &CallArgs,
    global: &Window,
    proto_id: PrototypeList::ID,
    creator: unsafe fn(SafeJSContext, HandleObject, *mut ProtoOrIfaceArray),
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
    )
    .is_ok()
}
