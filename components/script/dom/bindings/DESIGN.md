# The design of Garbage collected DOM

This is how Servo provides an object graph to SpiderMonkey's Garbage Collector.

## Construct
When Servo creates a Rusty DOM object the binding code creates a corresponding wrapper `JSObject` with SpiderMonkey. It’s produced and set to the Rusty object in `FooBinding::Wrap`.

In `FooBinding::Wrap`, the wrapper `JSObject` gets the pointer for the Rusty object to itself. At the same time the wrapper `JSObject` is set to the Rusty object’s `Reflector` field (all Rusty DOM objects have `dom::bindings::utils::Reflector` in their most basic fields). These steps are the “binding” work necessary to create the relationship between both objects.


## Trace object graph from SpiderMonkey GC
This is a tricky mechanism done with the help of the Rust compiler.
The outline is:

1. SpiderMonkey's GC calls `JSClass.trace` defined in `FooBinding` during the marking phase. This `JSClass` is the basis of each wrapper `JSObject`.
2. `JSClass.trace` calls `Foo::trace()` (an implementation of `JSTraceable`).
     This is typically derived via a #[jstraceable] annotation.
3. For all fields, `Foo::trace()`
   calls `trace()` on the field. For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
   `trace_reflector()`. Non-JS-managed types have an empty inline `trace()` method, achieved via `no_jsmanaged_fields!` or similar.
4. `trace_reflector()` fetches the reflector that is reachable from a Rust object and notifies it to the GC using JSTracer.
5. This operation continues for the rest of the graph.
6. Finally, the GC checks whether the Rust object lives or not from `JSObject`s which are held by Rust object.


## Destruct
When destructing DOM objects (wrapper JSObjects), SpiderMonkey calls the `JSClass.finalize()` which is basis of each wrapper `JSObject`. This method refers to `FooBinding::_finalize()`.

In the `_finalize()` function the pointer of the Rusty DOM object that is contained in the JSObject is unwrapped. It is then cast to a Rust owned pointer and assigned to an empty local variable. Thus we can destruct the Rusty object afterwards.


## Interact with Exact GC’s rooting
For supporting SpiderMonkey’s exact GC rooting, we introduce [some types](https://github.com/mozilla/servo/wiki/Using-DOM-types):

- `JS<T>` is used for the DOM typed field in a DOM type structure. The GC can trace them recursively while the enclosing DOM object (maybe root) is alive.
  - `LayoutJS<T>` is specialized `JS<T>` to use in layout. `Layout*Helper` must be implemented on this type to prevent calling methods from non layout code.
- `Temporary<T>` is used as a return value for functions returning a DOM type. They are rooted for the duration of their lifetime. But a retun value gets moved around which can break the LIFO ordering constraint. Thus we need to introduce `Root<T>`.
- `Root<T>` contains the pointer to `JSObject` which the represented DOM type has. SpiderMonkey's conservative stack scanner scans it's pointers and marks a pointed `JSObject` as GC root.
- `JSRef` is just a reference to the value rooted by `Root<T>`.
- `RootCollection` is used to dynamically check if rooting satisfies LIFO ordering, because SpiderMonkey's GC requires LIFO order (See also: [Exact Stack Rooting - Storing a GCPointer on the CStack](https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting)).
 - `MutHeap<T>` is a version of `Cell<T>` that is safe to use for internal mutability of  Spidermonkey heap objects like `JSVal` and `JS<T>`
