/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The code to expose the DOM to JavaScript through IDL bindings.
//!
//! Exposing a DOM object to JavaScript
//! ===================================
//!
//! As [explained earlier](../index.html#a-dom-object-and-its-reflector), the
//! implementation of an interface `Foo` involves two objects: the DOM object
//! (implemented in Rust) and the reflector (a `JSObject`).
//!
//! In order to expose the interface's members to the web, properties
//! corresponding to the operations and attributes are defined on an object in
//! the reflector's prototype chain or on the reflector itself.
//!
//! Typically, these properties are either value properties whose value is a
//! function (for operations) or accessor properties that have a getter and
//! optionally a setter function (for attributes, depending on whether they are
//! marked `readonly`).
//!
//! All these JavaScript functions are set up such that, when they're called,
//! they call a Rust function in the generated glue code. This glue code does
//! some sanity checks and [argument conversions](conversions/index.html), and
//! calls into API implementation for the DOM object.
//!
//! Rust reflections of WebIDL constructs
//! =====================================
//!
//! WebIDL members are turned into methods on the DOM object (static methods
//! for a static members and instance methods for regular members).
//!
//! The instance methods for an interface `Foo` are defined on a
//! `dom::bindings::codegen::Bindings::FooBindings::FooMethods` trait. This
//! trait is then implemented for `Foo`. (All methods take an `&self`
//! parameter, as pointers to DOM objects can be freely aliased.)
//!
//! The return type and argument types are determined
//! [as described below](#rust-reflections-of-webidl-types).
//! In addition to those, all methods that are
//! [allowed to throw](#throwing-exceptions)
//! will have the return value wrapped in
//! [`Fallible<T>`](error/type.Fallible.html).
//! Methods that use certain WebIDL types like `any` or `object` will get a
//! `*mut JSContext` argument prepended to the argument list. Static methods
//! will be passed a [`&GlobalScope`](../globalscope/struct.GlobalScope.html)
//! for the relevant global. This argument comes before the `*mut JSContext`
//! argument, if any.
//!
//! Rust reflections of WebIDL operations (methods)
//! -----------------------------------------------
//!
//! A WebIDL operation is turned into one method for every overload.
//! The first overload gets the base name, and consecutive overloads have an
//! underscore appended to the name.
//!
//! The base name of the Rust method is simply the name of the WebIDL operation
//! with the first letter converted to uppercase.
//!
//! Rust reflections of WebIDL attributes
//! -------------------------------------
//!
//! A WebIDL attribute is turned into a pair of methods: one for the getter and
//! one for the setter. A readonly attribute only has a getter and no setter.
//!
//! The getter's name is the name of the attribute with the first letter
//! converted to uppercase. It has `Get` prepended to it if the type of the
//! attribute is nullable or if the getter can throw.
//!
//! The method signature for the getter looks just like an operation with no
//! arguments and the attribute's type as the return type.
//!
//! The setter's name is `Set` followed by the name of the attribute with the
//! first letter converted to uppercase. The method signature looks just like
//! an operation with a void return value and a single argument whose type is
//! the attribute's type.
//!
//! Rust reflections of WebIDL constructors
//! ---------------------------------------
//!
//! A WebIDL constructor is turned into a static class method named
//! `Constructor`. The arguments of this method will be the arguments of the
//! WebIDL constructor, with a `&GlobalScope` for the relevant global prepended.
//! The return value of the constructor for MyInterface is exactly the same as
//! that of a method returning an instance of MyInterface. Constructors are
//! always [allowed to throw](#throwing-exceptions).
//!
//! Rust reflections of WebIDL types
//! --------------------------------
//!
//! The exact Rust representation for WebIDL types can depend on the precise
//! way that they're being used (e.g., return values and arguments might have
//! different representations).
//!
//! Optional arguments which do not have a default value are represented by
//! wrapping `Option<T>` around the representation of the argument type.
//! Optional arguments which do have a default value are represented by the
//! argument type itself, set to the default value if the argument was not in
//! fact passed in.
//!
//! Variadic WebIDL arguments are represented by wrapping a `Vec<T>` around the
//! representation of the argument type.
//!
//! See [the type mapping for particular types](conversions/index.html).
//!
//! Rust reflections of stringifiers
//! --------------------------------
//!
//! *To be written.*
//!
//! Rust reflections of legacy callers
//! ---------------------------------
//!
//! Legacy callers are not yet implemented.
//!
//! Throwing exceptions
//! ===================
//!
//! WebIDL methods, getters, and setters that need to throw exceptions need to
//! be explicitly marked as such with the `[Throws]`, `[GetterThrows]` and
//! `[SetterThrows]` custom attributes.
//!
//! `[Throws]` applies to both methods and attributes; for attributes it means
//! both the getter and the setter (if any) can throw. `[GetterThrows]` applies
//! only to attributes. `[SetterThrows]` applies only to writable attributes.
//!
//! The corresponding Rust methods will have the return value wrapped in
//! [`Fallible<T>`](error/type.Fallible.html). To throw an exception, simply
//! return `Err()` from the method with the appropriate [error value]
//! (error/enum.Error.html).

#![allow(unsafe_code)]
#![deny(missing_docs)]
#![deny(non_snake_case)]

pub(crate) mod buffer_source;
#[allow(dead_code)]
pub(crate) mod cell;
pub(crate) mod constructor;
pub(crate) mod conversions;
pub(crate) mod error;
pub(crate) mod frozenarray;
pub(crate) mod function;
pub(crate) mod import;
pub(crate) mod inheritance;
pub(crate) mod like;
pub(crate) mod principals;
pub(crate) mod proxyhandler;
pub(crate) mod refcounted;
pub(crate) mod reflector;
pub(crate) mod root;
pub(crate) mod serializable;
pub(crate) mod settings_stack;
pub(crate) mod str;
pub(crate) mod structuredclone;
pub(crate) mod trace;
pub(crate) mod transferable;
pub(crate) mod utils;
pub(crate) mod weakref;
pub(crate) mod xmlname;

pub(crate) use script_bindings::{callback, iterable, num};

/// Generated JS-Rust bindings.
#[allow(missing_docs, non_snake_case)]
pub(crate) mod codegen {
    pub(crate) mod DomTypeHolder {
        include!(concat!(env!("OUT_DIR"), "/DomTypeHolder.rs"));
    }
    pub(crate) use script_bindings::codegen::GenericBindings;
    #[allow(dead_code)]
    pub(crate) mod Bindings {
        include!(concat!(env!("OUT_DIR"), "/ConcreteBindings/mod.rs"));
    }
    pub(crate) mod InterfaceObjectMap {
        include!(concat!(env!("OUT_DIR"), "/InterfaceObjectMap.rs"));
    }
    pub(crate) mod ConcreteInheritTypes {
        include!(concat!(env!("OUT_DIR"), "/ConcreteInheritTypes.rs"));
    }
    pub(crate) mod StubbedInterfaces {
        include!(concat!(env!("OUT_DIR"), "/StubbedInterfaces.rs"));
    }
    pub(crate) use script_bindings::codegen::{PrototypeList, RegisterBindings};
    #[allow(dead_code)]
    pub(crate) mod UnionTypes {
        include!(concat!(env!("OUT_DIR"), "/UnionTypes.rs"));
    }
}
