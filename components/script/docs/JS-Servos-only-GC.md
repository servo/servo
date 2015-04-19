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
'reflector' object in the JavaScript VM that holds a reference to the
underlying low-level object. If the JavaScript garbage collector determines
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
to the `Element` reflector in JavaScript. The C++ refcounting will never
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
references, it might free a node that is still in use. It's essential for both
security and programmer convenience that we get rid of this manual listing of
fields.

Rust has a notion of [traits][traits], which are similar to
[type classes][typeclasses] in Haskell or interfaces in many object-oriented
languages. For example, we can create a `HasArea` trait:

[traits]: https://doc.rust-lang.org/book/traits.html XXX
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
pub trait JSTraceable {
    fn trace(&self, trc: *mut JSTracer);
}
```

Any type which can be traced will provide a `trace` method. We will implement
this method with a [custom attribute][custom] `#[jstraceable]`, or
`#[dom_struct]` which implies it.

XXX: something about the implementation?

[custom]: XXX

Let's look at [Servo's implementation][document-rs] of the DOM's
[`Document`][document-mdn] interface:

[document-rs]: XXX
[document-mdn]: https://developer.mozilla.org/en-US/docs/Web/API/document

```rust
#[dom_struct]
pub struct Document {
    node: Node,
    window: JS<Window>,
    is_html_document: bool,
    ...
}
```

Note the difference between the `node` and `window` fields above. In the
[object hierarchy of the DOM spec][document-spec], every `Document` is also a
`Node`. Rust doesn't have inheritance for data types, so we implement this by
storing a `Node` struct within a `Document` struct. As in C++, the fields of
`Node` are included in-line with the fields of `Document`, without any pointer
indirection, and the auto-generated `trace` method will visit them as well.

[document-spec]: http://dom.spec.whatwg.org/#interface-document

A `Document` also has an associated `Window`, but this is not an 'is-a'
relationship. The `Document` just has a pointer to a `Window`, one of many
pointers to that object, which can live in native DOM data structures or in
JavaScript objects. These are precisely the pointers we need to tell the
garbage collector about. We do this with a
[custom type for traced pointers: `JS<T>`][js] (for example, the `JS<Window>`
above). The implementation of [`trace` for `JS<T>`][trace-js] is not
auto-generated; this is where we actually call the SpiderMonkey trace hooks.

[js]: http://doc.servo.org/script/dom/bindings/js/struct.JS.html
[trace-js]: XXX

Lifetime checking for safe rooting
==================================

The Rust code in Servo needs to pass DOM object pointers as function arguments,
store DOM object pointers in local variables, and so forth. We need to register
these additional temporary references as [roots][roots] in the garbage
collector's reachability analysis. If we touch an object from Rust when it's
not rooted, that could introduce a use-after-free vulnerability.

[roots]: https://en.wikipedia.org/wiki/Tracing_garbage_collection#Reachability_of_an_object

To make this happen, we need to expand our repertoire of GC-managed pointer
types. We already talked about `JS<T>`, which represents a traced reference
between two GC-managed Rust objects. These are not rooted; the garbage
collector only knows about them when `trace` reaches one as part of the tracing
process.

When we want to use a DOM object from Rust code, we call the [`root`][js-root]
method on `JS<T>`. For example:

[js-root]: http://doc.servo.org/script/dom/bindings/js/struct.JS.html#method.root XXX

```rust
fn load_anchor_href(&self, href: DOMString) {
    let window = self.window.root();
    window.load_url(href);
}
```

The `root` method returns a [`Root<T>`][root], which is stored in a
stack-allocated local variable. When the `Root<T>` is destroyed at the end of
the function, its destructor will un-root the DOM object. This is an example of
the [RAII idiom][raii], which Rust inherits from C++.

[root]: http://doc.servo.org/script/dom/bindings/js/struct.Root.html
[raii]: https://en.wikipedia.org/wiki/Resource_Acquisition_Is_Initialization

Of course, a DOM object might make its way through many function calls and
local variables before we're done with it. We want to avoid the cost of telling
SpiderMonkey about each and every step. Instead, we have another type
[`JSRef<T>`][jsref], which represents a pointer to a GC-managed object that is
already rooted elsewhere. Unlike `Root<T>`, `JSRef<T>` can be copied at
negligible cost.

[jsref]: http://doc.servo.org/script/dom/bindings/js/struct.JSRef.html

We shouldn't un-root an object if it's still reachable through `JSRef<T>`, so
it's important that a `JSRef<T>` can't outlive its originating `Root<T>`.
Situations like this are common in C++ as well. No matter how smart your smart
pointer is, you can take a bare reference to the contents and then erroneously
use that reference past the lifetime of the smart pointer.

Rust solves this problem with a compile-time [lifetime checker][lifetimes].
The type of a reference includes the region of code over which it is valid. In
most cases, lifetimes are [inferred][ti] and don't need to be written out in
the source code. Inferred or not, the presence of lifetime information allows
the compiler to reject use-after-free and other dangerous bugs.

[lifetimes]: http://doc.rust-lang.org/guide-lifetimes.html XXX
[ti]: http://en.wikipedia.org/wiki/Type_inference

Not only do lifetimes protect Rust's built-in reference type, we can use them
in our own data structures as well. `JSRef` is actually [defined][jsref-rs] as

[jsref-rs]: XXX

```rust
pub struct JSRef<'a, T> {
    ptr: NonZero<*const T>,
    chain: PhantomData<&'a ()>,
}
```

`T` is the familiar type variable, representing the type of DOM structure we're
pointing to, e.g. `Window`. The somewhat odd syntax `'a` is a
[lifetime variable][named-lifetime], representing the region of code in which
that object is rooted. Crucially, this lets us write a [method][root-r] on
`Root` with the following signature:

[named-lifetime]: http://doc.rust-lang.org/guide-lifetimes.html#named-lifetimes
[root-r]: XXX

```rust
pub fn r<'a>(&'a self) -> JSRef<'a, T> {
    ...
```

What this syntax means is:

- **`<'a>`**: 'for any lifetime `'a`',
- **`(&'a self)`**: 'take a reference to a `Root` which is valid over lifetime `'a`',
- **`-> JSRef<'a, T>`**: 'return a `JSRef` whose lifetime parameter is set to `'a`'.

The final piece of the puzzle is that we put a [marker][phantom] in the `JSRef`
type saying that it's only valid for the lifetime of a reference `&'a T`. This
is how we extend the lifetime system to enforce our application-specific
property about garbage collector rooting. If we try to compile something like
this:

[phantom]: http://doc.rust-lang.org/std/marker/struct.PhantomData.html

```rust
fn bogus_get_window<'a>(&self) -> JSRef<'a, Window> {
    let window = self.window.root();
    window.r()  // return the JSRef
}
```

we get an error:

XXX
<pre class="sourceCode">`document.rs:199:9: 199:15 <span style="color: red">error:</span> <b>`window` does not live long enough</b>
document.rs:199     window.root_ref()
                    <span style="color: red">^~~~~~</span>
document.rs:197:57: 200:6 <span style="color: green">note:</span> <b>reference must be valid for
    the lifetime "a as defined on the block at 197:56...</b>
document.rs:197 fn bogus_get_window<"a>(&amp;self) -> JSRef<"a, Window> {
document.rs:198     let window = self.window.root();
document.rs:199     window.root_ref()
document.rs:200 }
document.rs:197:57: 200:6 <span style="color: green">note:</span> <b>...but borrowed value is only
    valid for the block at 197:56</b>
document.rs:197 fn bogus_get_window<"a>(&amp;self) -> JSRef<"a, Window> {
document.rs:198     let window = self.window.root();
document.rs:199     window.root_ref()
document.rs:200 }</pre>

We also implement the [`Deref` trait][deref] for `JSRef<T>`. This allows us to
access fields of the underlying type `T` through a `JSRef<T>`. Because `JS<T>`
does *not* implement `Deref` or otherwise provide access to the underlying
pointer, we have to root an object before using it.

[deref]: http://doc.rust-lang.org/std/ops/trait.Deref.html

The DOM methods of `Window` (for example) are defined in a trait which is
[implemented][windowmethods] for `JSRef<Window>`. This ensures that the
`self` pointer is rooted for the duration of the method call, which would not
be guaranteed if we implemented the methods on `Window` directly.

XXX: this is not really true, is it?

[windowmethods]: XXX

You can check out the [`js` module's documentation][js-docs] for more details
that didn't make it into this document.

[js-docs]: http://doc.servo.org/script/dom/bindings/js/index.html

Custom static analysis
======================

To recap, the safety of our system depends on two major parts:

- The auto-generated `trace` methods ensure that SpiderMonkey's garbage
  collector can see all of the references between DOM objects.
- The implementation of `Root<T>` and `JSRef<T>` guarantees that we can't use a
  DOM object from Rust without telling SpiderMonkey about our temporary
  reference.

But there's a hole in this scheme. We could copy an unrooted pointer — a
`JS<T>` — to a local variable on the stack, and then at some later point, root
it and use the DOM object. In the meantime, SpiderMonkey's garbage collector
won't know about that `JS<T>` on the stack, so it might free the DOM object.
To really be safe, we need to make sure that `JS<T>` *only* appears in places
where it will be traced, such as DOM structs, and never in local variables,
function arguments, and so forth.

// XXX: this gives me a crazy idea: what if we made JS<T> !Copy?

This rule doesn't correspond to anything that already exists in Rust's type
system. Fortunately, the Rust compiler can load 'lint plugins' providing custom
static analysis. These basically take the form of new compiler warnings,
although in this case we set the default severity to 'error'.

[lints]: https://github.com/rust-lang/rust/pull/15024 (better link?)

We have already [implemented a plugin][js-lint] which simply forbids `JS<T>`
from appearing at all. Because lint plugins are part of the usual
[warnings infrastructure][warnings], we can use the `allow` attribute in places
where it's okay to use `JS<T>`, like DOM struct definitions and the
implementation of `JS<T>` itself.

[js-lint]: https://github.com/kmcallister/servo/commit/c20b50bbbbcdc8ce3551adbc1e039a727cf89995 (better link?)
[warnings]: http://doc.rust-lang.org/rust.html#lint-check-attributes (better link?)

Our plugin looks at every place where the code mentions a type. Remarkably,
this adds only a fraction of a second to the compile time for Servo's largest
subcomponent, as Rust compile times are dominated by [LLVM][llvm]'s back-end
optimizations and code generation. The current version of the plugin is very
simple and will miss some mistakes, like storing a struct containing `JS<T>` on
the stack. (XXX does it still?) However, lint plugins run at a late stage of
compilation and have access to full compiler internals, including the results
of type inference. So we can make the plugin incrementally more sophisticated
in the future.

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

It's an open question how our garbage-collected DOM will perform compared to
a traditional reference-counted DOM. The [Blink][blink] team has performed
[similar experiments][blink-gc] (XXX last I checked they just used a separate
GC, which isn't similar at all!), but they don't have Servo's luxury of
starting from a clean slate and using a cutting-edge language. We expect the
biggest gains will come when we move to allocating DOM objects within the
JavaScript reflectors themselves. Since the reflectors need to be traced no
matter what, this will reduce the cost of managing native DOM structures to
almost nothing.

[blink]: http://www.chromium.org/blink
[blink-gc]: http://www.chromium.org/blink/blink-gc

XXX: this doesn't say anything about conservative stack scanning! Should it?
