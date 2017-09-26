% JavaScript: Servo's only garbage collector

A web browser's purpose in life is to mediate interaction between a user and
an application (which we somewhat anachronistically call a 'document').
Users expect a browser to be fast and responsive, so the core layout and
rendering algorithms are typically implemented in low-level native code.
At the same time, JavaScript code in the document can perform complex
modifications through the [Document Object Model][DOM].
This means the browser's representation of a document in memory is a
cross-language data structure, bridging the gap between low-level native code
and the high-level, garbage-collected world of JavaScript.

We're taking this as another opportunity in the Servo project to advance the
state of the art. We have a new approach for DOM memory management, and we get
to use some of the Rust language's exciting features, like auto-generated trait
implementations, lifetime checking, and custom static analysis plugins.

[DOM]: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model

Memory management for the DOM
=============================

It's essential that we never destroy a DOM object while it's still reachable
from either JavaScript or native code — such [use-after-free bugs][uaf] often
produce exploitable security holes. To solve this problem, most existing
browsers use [reference counting][refcounting] to track the pointers between
underlying low-level DOM objects. When JavaScript retrieves a DOM object
(through [`getElementById`][gEBI] for example), the browser builds a
'reflector' object in the JavaScript virtual machine that holds a reference to
the underlying low-level object. If the JavaScript garbage collector determines
that a reflector is no longer reachable, it destroys the reflector and
decrements the reference count on the underlying object.

[uaf]: http://cwe.mitre.org/data/definitions/416.html
[refcounting]: https://en.wikipedia.org/wiki/Reference_counting
[gEBI]: https://developer.mozilla.org/en-US/docs/Web/API/document.getElementById

This solves the use-after-free issue. But to keep users happy, we also need to
keep the browser's memory footprint small. This means destroying objects as
soon as they are no longer needed. Unfortunately, the cross-language
'reflector' scheme introduces a major complication.

Consider a C++ `Element` object which holds a reference-counted pointer to an
`Event`:

```cpp
struct Element {
    RefPtr<Event> mEvent;
};
```

Now suppose we add an event handler to the element from JavaScript:

```js
elem.addEventListener("load", function(event) {
    event.originalTarget = elem;
});
```

When the event fires, the handler adds a property on the `Event` which points
back to the `Element`. We now have a cross-language reference cycle, with an
`Element` pointing to an `Event` within C++, and an `Event` reflector pointing
to the `Element` reflector in JavaScript. The C++ reference counting will never
destroy a cycle, and the JavaScript garbage collector can't trace through the
C++ pointers, so these objects will never be freed.

Existing browsers resolve this problem in several ways. Some do nothing, and
leak memory. Some try to manually break possible cycles, by nulling out
`mEvent` for example. And some implement a [cycle collection][cc] algorithm on
top of reference counting.

[cc]: https://developer.mozilla.org/en-US/docs/Interfacing_with_the_XPCOM_cycle_collector

None of these solutions are particularly satisfying, so we're trying something
new in Servo by choosing not to reference count DOM objects at all. Instead,
we give the JavaScript garbage collector full responsibility for managing those
native-code DOM objects. This requires a fairly complex interaction between
Servo's Rust code and the [SpiderMonkey][SM] garbage collector, which is
written in C++. Fortunately, Rust provides some cool features that let us build
this in a way that's fast, secure, and maintainable.

[SM]: https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey

Auto-generating field traversals
================================

How will the garbage collector find all the references between DOM objects? In
[Gecko]'s cycle collector this is done with a lot of hand-written annotations,
e.g.:

[Gecko]: https://developer.mozilla.org/en-US/docs/Mozilla/Gecko

```cpp
NS_IMPL_CYCLE_COLLECTION(nsFrameLoader, mDocShell, mMessageManager)
```

This macro describes which members of a C++ class should be added to a graph
of potential cycles. Forgetting an entry can produce a memory leak. In Servo
the consequences would be even worse: if the garbage collector can't see all
references, it might free an object that is still in use. It's essential for
both security and programmer convenience that we get rid of this manual listing
of fields.

Rust has a notion of [traits][traits], which are similar to
[type classes][typeclasses] in Haskell or interfaces in many object-oriented
languages. For example, we can create a `HasArea` trait:

[traits]: https://doc.rust-lang.org/book/traits.html
[typeclasses]: http://learnyouahaskell.com/types-and-typeclasses

```rust
trait HasArea {
    fn area(&self) -> f64;
}
```

Any type implementing the `HasArea` trait will provide a method named `area`
that takes a value of the type (by reference, hence `&self`) and returns a
floating point number. In other words, the `HasArea` trait describes any type
which has an area, and the trait provides a way to get that object's area.

Now let's look at the `JSTraceable` trait, which we use for tracing:

```rust
pub unsafe trait JSTraceable {
    unsafe fn trace(&self, trc: *mut JSTracer);
}
```

Any type which can be traced will provide a `trace` method. We will implement
this method with a custom `derive` target `#[derive(JSTraceable)]`,
or a custom attribute `#[dom_struct]` which implies it.

Let's look at [Servo's implementation][document-rs] of the DOM's
[`Document`][document-mdn] interface:

[document-rs]: https://github.com/servo/servo/blob/master/components/script/dom/document.rs
[document-mdn]: https://developer.mozilla.org/en-US/docs/Web/API/document

```rust
use dom_struct::dom_struct;

#[dom_struct]
pub struct Document {
    node: Node,
    window: Dom<Window>,
    is_html_document: bool,
    ...
}
```

Note the difference between the `node` and `window` fields above. In the
object hierarchy of the DOM (as defined in
[the DOM specification][document-spec]), every `Document` is also a `Node`.
Rust doesn't have inheritance for data types, so we implement this by
storing a `Node` struct within a `Document` struct. As in C++, the fields of
`Node` are included in-line with the fields of `Document`, without any pointer
indirection, and the auto-generated `trace` method will visit them as well.

[document-spec]: http://dom.spec.whatwg.org/#interface-document

A `Document` also has an associated `Window`, but this is not an 'is-a'
relationship. The `Document` just has a pointer to a `Window`, one of many
pointers to that object, which can live in native DOM data structures or in
JavaScript objects. These are precisely the pointers we need to tell the
garbage collector about. We do this with a
[custom type for traced pointers: `Dom<T>`][dom] (for example, the `Dom<Window>`
above). The implementation of `trace` for `Dom<T>` is not auto-generated; this
is where we actually call the SpiderMonkey trace hooks:

[dom]: http://doc.servo.org/script/dom/bindings/root/struct.Dom.html

```rust
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    unsafe {
        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter_ = None;
        (*tracer).debugPrintIndex_ = !0;
        (*tracer).debugPrintArg_ = name.as_ptr() as *const libc::c_void;
        debug!("tracing reflector {}", description);
        JS_CallUnbarrieredObjectTracer(tracer, reflector.rootable(),
                                       GCTraceKindToAscii(JSGCTraceKind::JSTRACE_OBJECT));
    }
}

impl<T: DomObject> JSTraceable for Dom<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", unsafe { (**self.ptr).reflector() });
    }
}
```

This call will also update the pointer to the reflector, if it was
[moved][moving-gc].

[moving-gc]: https://en.wikipedia.org/wiki/Tracing_garbage_collection#Moving_vs._non-moving

Lifetime checking for safe rooting
==================================

The Rust code in Servo needs to pass pointers to DOM objects as function
arguments, store pointers to DOM objects in local variables, and so forth.
We need to make sure that the DOM objects behind these additional pointers are
kept alive by the garbage collector while we use them. If we touch an object
from Rust when the garbage collector is not aware of it, that could introduce a
use-after-free vulnerability.

We use Rust's built-in reference type (`&T`) for pointers to DOM objects that
are known to the garbage collector. Note that such a reference is nothing more
than a pointer in the compiled code; the reference does not by itself signal
anything to the garbage collector. As a pointer to a DOM object might make its
way through many function calls and local variables before we're done with it,
we need to avoid the cost of telling the garbage collector about each and every
step along the way.

Such a reference can be obtained in different ways. For example, DOM code is
often called from JavaScript through [IDL-based bindings][bindings]. In this
case, the bindings code constructs a [root][gc-root] for any involved DOM
objects.

[bindings]: http://doc.servo.org/script/dom/bindings/index.html
[gc-root]: https://en.wikipedia.org/wiki/Tracing_garbage_collection#Reachability_of_an_object

Another common situation is creating a stack-local root manually. For this
purpose, we have a [`DomRoot<T>`][root] struct. When the `DomRoot<T>` is destroyed,
typically at the end of the function (or block) where it was created, its
destructor will un-root the DOM object. This is an example of the
[RAII idiom][raii], which Rust inherits from C++.
`DomRoot<T>` structs are primarily returned from [`T::new` functions][new] when
creating a new DOM object.
In some cases, we need to use a DOM object longer than the reference we
received allows us to; the [`DomRoot::from_ref` associated function][from-ref]
allows creating a new `DomRoot<T>` struct in that case.

[root]: http://doc.servo.org/script/dom/bindings/root/struct.DomRoot.html
[raii]: https://en.wikipedia.org/wiki/Resource_Acquisition_Is_Initialization
[new]: http://doc.servo.org/script/dom/index.html#construction
[from-ref]: http://doc.servo.org/script/dom/bindings/root/struct.DomRoot.html#method.from_ref

We can then obtain a reference from the `DomRoot<T>` through Rust's built-in
[`Deref` trait][deref], which exposes a method `deref` with the following
signature:

```rust
pub fn deref<'a>(&'a self) -> &'a T {
    ...
```

What this syntax means is:

- **`<'a>`**: 'for any lifetime `'a`',
- **`(&'a self)`**: 'take a reference to a `DomRoot` which is valid over lifetime `'a`',
- **`-> &'a T`**: 'return a reference whose lifetime is limited to `'a`'.

This allows us to call methods and access fields of the underlying type `T`
through a `DomRoot<T>`.

[deref]: https://doc.rust-lang.org/std/ops/trait.Deref.html

A third way to obtain a reference is from the `Dom<T>` struct we encountered
earlier. Whenever we have a reference to a `Dom<T>`, we know that the DOM struct
that contains it is already rooted, and thus that the garbage collector is
aware of the `Dom<T>`, and will keep the DOM object it points to alive.
This allows us to implement the `Deref` trait on `Dom<T>` as well.

The correctness of these APIs is heavily dependent on the fact that the
reference cannot outlive the smart pointer it was retrieved from, and the fact
that the smart pointer cannot be modified while the reference extracted from it
exists.

Situations like this are common in C++ as well. No matter how smart your smart
pointer is, you can take a bare pointer to the contents and then erroneously
use that pointer past the lifetime of the smart pointer.

Rust guarantees those semantics for the built-in reference type with a
compile-time [lifetime checker][lifetimes]. The type of a reference includes
the region of code over which it is valid. In most cases, lifetimes are
[inferred][ti] and don't need to be written out in the source code. Inferred or
not, the presence of lifetime information allows the compiler to reject
use-after-free and other dangerous bugs.

[lifetimes]: https://doc.rust-lang.org/book/lifetimes.html
[ti]: https://en.wikipedia.org/wiki/Type_inference

You can check out the [`root` module's documentation][root-docs] for more details
that didn't make it into this document.

[root-docs]: http://doc.servo.org/script/dom/bindings/root/index.html

Custom static analysis
======================

To recapitulate, the safety of our system depends on two major parts:

- The auto-generated `trace` methods ensure that SpiderMonkey's garbage
  collector can see all of the references between DOM objects.
- The implementation of `DomRoot<T>` guarantees that we can't use a DOM object
  from Rust without telling SpiderMonkey about our temporary reference.

But there's a hole in this scheme. We could copy an unrooted pointer — a
`Dom<T>` — to a local variable on the stack, and then at some later point, root
it and use the DOM object. In the meantime, SpiderMonkey's garbage collector
won't know about that `Dom<T>` on the stack, so it might free the DOM object.
To really be safe, we need to make sure that `Dom<T>` *only* appears in places
where it will be traced, such as DOM structs, and never in local variables,
function arguments, and so forth.

This rule doesn't correspond to anything that already exists in Rust's type
system. Fortunately, the Rust compiler can load 'lint plugins' providing custom
static analysis. These basically take the form of new compiler warnings,
although in this case we set the default severity to 'error'. There is more
information about lints in section 4.7 of the paper [<cite>Experience Report:
Developing the Servo Web Browser Engine using Rust</cite>][lints].

[lints]: http://arxiv.org/pdf/1505.07383v1.pdf

We have already [implemented a plugin][js-lint] which effectively forbids
`Dom<T>` from appearing on the [stack][stack]. Because lint plugins are part of
the usual [warnings infrastructure][warnings], we can use the `allow` attribute
in places where it's okay to use `Dom<T>`, like DOM struct definitions and the
implementation of `Dom<T>` itself.

[js-lint]: http://doc.servo.org/plugins/lints/unrooted_must_root/struct.UnrootedPass.html
[stack]: https://en.wikipedia.org/wiki/Stack-based_memory_allocation
[warnings]: https://doc.rust-lang.org/book/compiler-plugins.html#lint-plugins

Our plugin looks at every place where the code mentions a type. Remarkably,
this adds only a fraction of a second to the compile time for Servo's largest
subcomponent, as Rust compile times are dominated by [LLVM][llvm]'s back-end
optimizations and code generation.

[llvm]: http://llvm.org/

In the end, the plugin won't necessarily catch every mistake. It's hard to
achieve full [soundness][soundness] with ad-hoc extensions to a type system.
As the name 'lint plugin' suggests, the idea is to catch common mistakes at a
low cost to programmer productivity. By combining this with the lifetime
checking built in to Rust's type system, we hope to achieve a degree of
security and reliability far beyond what's feasible in C++. Additionally, since
the checking is all done at compile time, there's no penalty in the generated
machine code.

[soundness]: https://en.wikipedia.org/wiki/Soundness

Conclusion and future work
==========================

It's an open question how our garbage-collected DOM will perform compared to
a traditional reference-counted DOM. The [Blink][blink] team has also performed
[experiments with garbage collection DOM objects][blink-gc], but they don't
have Servo's luxury of starting from a clean slate and using a cutting-edge
language.

[blink]: http://www.chromium.org/blink
[blink-gc]: http://www.chromium.org/blink/blink-gc

In the future, we will likely attempt to merge the allocations of DOM objects
into the allocations of their JavaScript reflectors. This could produce
additional gains, since the reflectors need to be traced no matter what.
However, as the garbage collector can move reflectors in memory, this will make
the [use of Rust's built-in references][refs] infeasible.

[refs]: #lifetime-checking-for-safe-rooting
