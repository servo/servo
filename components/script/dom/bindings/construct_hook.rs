use std::env::args;

pub fn get_global_scope(cx: *mut JSContext, vp: *mut JSVal, argc: u32, exposureSet: ExposureSet) -> GlobalScope {
    let cx = SafeJSContext::from_ptr(cx);
    let args = CallArgs::from_vp(vp, argc);
    let global = GlobalScope::from_object(JS_CALLEE(*cx, vp).to_object());

    if exposureSet.len() == 1 {
        let globalType = list(exposure_set)[0];
        return DomRoot::downcast::<dom::types::global_type>(global).unwrap();
    }
    return global;
}

pub unsafe fn handle_constructor(cx: *mut JSContext, args: &CallArgs, global: &GlobalScope, descriptor: &Descriptor, constructor: &Constructor) -> bool {
    if constructor.isHTMLConstructor() {
        let signatures = constructor.signatures();
        assert!(signatures.len() == 1);
        match descriptor.name.as_str() {
            "Type1" => {
                let callResult = dom::bindings::htmlconstructor::call_html_constructor::<dom::types::Type1>(
                    cx,
                    args,
                    global,
                    PrototypeList::ID::Type1,
                    CreateInterfaceObjects,
                );
            }
            "Type2" => {
                let callResult = dom::bindings::htmlconstructor::call_html_constructor::<dom::types::Type2>(
                    cx,
                    args,
                    global,
                    PrototypeList::ID::Type2,
                    CreateInterfaceObjects,
                );
            }
            _ => panic!("Unsupported type: {}", descriptor.name),
        }
        return callResult;
    } else {
        let ctorName = GetConstructorNameForReporting(descriptor, constructor);
        if !callargs_is_constructing(args) {
            throw_constructor_without_new(*cx, ctorName);
            return false;
        }
        rooted!(in(*cx) let mut desired_proto = ptr::null_mut::<JSObject>());
        let proto_result = get_desired_proto(
            cx,
            args,
            PrototypeList::ID::MakeNativeName(descriptor.name),
            CreateInterfaceObjects,
            desired_proto.handle_mut(),
        );
        assert!(proto_result.is_ok());
        if proto_result.is_err() {
            return false;
        }
        let name = constructor.identifier.name;
        let nativeName = MakeNativeName(descriptor.binaryNameFor(name));
        let args = ["global", "Some(desired_proto.handle())"];
        let constructorCall = CGMethodCall(args, nativeName, true, descriptor, constructor);
        return constructorCall;
    }
}