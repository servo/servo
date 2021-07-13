use js::glue::{
    CreateRustJSPrincipals, DestroyRustJSPrincipals, GetRustJSPrincipalsPrivate,
    JSPrincipalsCallbacks,
};
use js::jsapi::JSPrincipals;
use servo_url::MutableOrigin;

// TODO: RAII ref-counting
pub struct ServoJSPrincipals(pub *mut JSPrincipals);

impl ServoJSPrincipals {
    pub fn new(origin: &MutableOrigin) -> Self {
        let private: Box<MutableOrigin> = Box::new(origin.clone());
        Self(unsafe { CreateRustJSPrincipals(&PRINCIPALS_CALLBACKS, Box::into_raw(private) as _) })
    }

    pub unsafe fn origin(&self) -> MutableOrigin {
        let origin = GetRustJSPrincipalsPrivate(self.0) as *mut MutableOrigin;
        (*origin).clone()
    }
}

pub unsafe extern "C" fn destroy_servo_jsprincipal(principals: *mut JSPrincipals) {
    Box::from_raw(GetRustJSPrincipalsPrivate(principals) as *mut MutableOrigin);
    DestroyRustJSPrincipals(principals);
}

const PRINCIPALS_CALLBACKS: JSPrincipalsCallbacks = JSPrincipalsCallbacks {
    write: None,
    isSystemOrAddonPrincipal: Some(principals_is_system_or_addon_principal),
};

unsafe extern "C" fn principals_is_system_or_addon_principal(_: *mut JSPrincipals) -> bool {
    false
}

//TODO is same_origin_domain equivalent to subsumes for our purposes
pub unsafe extern "C" fn subsumes(obj: *mut JSPrincipals, other: *mut JSPrincipals) -> bool {
    let obj = ServoJSPrincipals(obj);
    let other = ServoJSPrincipals(other);
    let obj_origin = obj.origin();
    let other_origin = other.origin();
    obj_origin.same_origin_domain(&other_origin)
}
