/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![macro_use]


macro_rules! check_ptr_exist {
    ($var:expr, $member:ident) => (
        unsafe { (*CefWrap::to_c($var)).$member.is_some() }
    );
}

// Provides the implementation of a CEF class. An example follows:
//
//    struct ServoCefThing {
//        ...
//    }
//
//    cef_class_impl! {
//        ServoCefThing : CefThing, cef_thing_t {
//            // Declare method implementations using the *C* API. (This may change later, once we
//            // have associated types in Rust.)
//            //
//            // Note that if the method returns unit, you must write `-> ()` explicitly. This is
//            // due to limitations of Rust's macro system.
//            fn foo(&this, a: isize, b: *mut cef_other_thing_t) -> () {
//                // Inside here, `a` will have type `isize`, and `b` will have the type
//                // `CefOtherThing` -- i.e. the Rust-wrapped version of `cef_other_thing_t`.
//                ...
//            }
//
//            fn bar(&this, a: isize) -> *mut cef_other_thing_t {
//                // Return types are automatically unwrapped from the Rust types (e.g.
//                // `CefOtherThing`) into the corresponding C types (e.g. `*mut
//                // cef_other_thing_t`).
//                let x: CefOtherThing = ...;
//                x
//            }
//        }
//    }
macro_rules! cef_class_impl(
    ($class_name:ident : $interface_name:ident, $c_interface_name:ident {
        $(
            fn $method_name:ident ( & $method_this:ident ,
                                   $( $method_arg_name:ident : $c_method_arg_type:ty ,)* )
                                   -> $method_return_type:ty {$method_body:block}
        )*
    }) => (
        full_cef_class_impl! {
            $class_name : $interface_name, $c_interface_name {
                $(fn $method_name(&$method_this,
                                  $($method_arg_name : $c_method_arg_type [$c_method_arg_type],)*)
                                  -> $method_return_type {
                    $method_body
                })*
            }
        }
    )
);

macro_rules! full_cef_class_impl(
    ($class_name:ident : $interface_name:ident, $c_interface_name:ident {
        $(
            fn $method_name:ident ( & $method_this:ident ,
                                   $( $method_arg_name:ident : $c_method_arg_type:ty [$method_arg_type:ty] ,)* )
                                   -> $method_return_type:ty {$method_body:block}
        )*
    }) => (
        impl $class_name {
            pub fn as_cef_interface(self) -> $interface_name {
                let cef_object = unsafe {
                    // Calculate the offset of the reference count. This is the size of the
                    // structure.
                    let null: *const $c_interface_name = ::std::ptr::null();
                    let offset: *const u32 = &(*null).ref_count;
                    let size = (offset as ::libc::size_t) - (null as ::libc::size_t);
                    $interface_name::from_c_object_addref(
                        ::eutil::create_cef_object::<$c_interface_name,$class_name>(size))
                };
                unsafe {
                    $((*cef_object.c_object()).$method_name = Some($method_name as extern "C" fn(*mut $c_interface_name, $($c_method_arg_type,)*) -> $method_return_type);)*
                    let extra_slot =
                        ::std::mem::transmute::<&mut u8,
                                                &mut $class_name>(&mut (*cef_object.c_object())
                                                                                   .extra);
                    ::std::ptr::write(extra_slot, self);
                }
                cef_object
            }
        }

        $(
            extern "C" fn $method_name(raw_this: *mut $c_interface_name,
                                       $($method_arg_name: $c_method_arg_type),*)
                                       -> $method_return_type {
                let $method_this = unsafe {
                    $interface_name::from_c_object_addref(raw_this)
                };
                $(
                    let $method_arg_name: $method_arg_type = unsafe {
                        ::wrappers::CefWrap::to_rust($method_arg_name)
                    };
                )*
                ::wrappers::CefWrap::to_c($method_body)
            }
        )*

        impl ::eutil::Downcast<$class_name> for $interface_name {
            fn downcast(&self) -> &$class_name {
                unsafe {
                    ::std::mem::transmute::<&u8,&$class_name>(&(*self.c_object()).extra)
                }
            }
        }
    )
);

macro_rules! cef_static_method_impls(
    (
        $(
            fn $method_name:ident ( $($method_arg_name:ident : $method_arg_type:ty ),* )
                                   -> $method_return_type:ty {$method_body:block}
        )*
    ) => (
        $(
            #[no_mangle]
            pub extern "C" fn $method_name($($method_arg_name: $method_arg_type),*)
                                           -> $method_return_type {
                $(
                    let $method_arg_name = unsafe {
                        ::wrappers::CefWrap::to_rust($method_arg_name)
                    };
                )*
                ::wrappers::CefWrap::to_c($method_body)
            }
        )*
    )
);

macro_rules! cef_stub_static_method_impls(
    (
        $(
            fn $method_name:ident ( $($method_arg_name:ident : $method_arg_type:ty ),* )
                                   -> $method_return_type:ty
        )*
    ) => (
        $(
            #[no_mangle]
            pub extern "C" fn $method_name($(_: $method_arg_type),*)
                                           -> $method_return_type {
                panic!("unimplemented static method: {}", stringify!($method_name))
            }
        )*
    )
);
