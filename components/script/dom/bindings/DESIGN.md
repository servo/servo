# The design of Garbage collected DOM

These are how Servo provides an object graph to SpiderMonkey's Garbage Collection.

## Construct
When Servo creates a Rusty DOM object, the binding code creates a wrapper `JSObject` with SpiderMonkey, is correspond to each Rusty DOM Object. It’s produced and set to the Rusty object in `FooBinding::Wrap`.

In `FooBinding::Wrap`, the wrapper JSObject gets the pointer for Rusty Object to itself. And the same time,  the wrapper `JSObject` are set to the Rusty Object’s `Reflector` field (All Rusty DOM objects have `dom::bindings::utils::Reflector` in their most basis field). These step are the “binding” work to create the relationship of both objects.


## Trace object graph from SpiderMonkey GC.
This is very tricky and magically mechanism helped by Rust Compiler.
The outline is:

1. SpiderMonkey's GC calls `JSClass.trace` defined in `FooBinding` when marking phase. This JSClass is basis of each wrapper JSObject.
2. `JSClass.trace` calls `Foo::trace()` (an implementation of `JSTraceable`).
     This is typically derived via a #[jstraceable] annotation
3. For all fields, `Foo::trace()`
   calls `trace()` on the field. For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
   `trace_reflector()`. Non-JS-managed types have an empty inline `trace()` method, achieved via `untraceable!` or similar.
4. `trace_reflector()` fetches the reflector that is reachable from a Rust object, and notifies it to the GC with using JSTracer.
5. This operation continues to the end of the graph.
6. Finally, GC gets whether Rust object lives or not from JSObjects which is hold by Rust object.


## Destruct
When destructing DOM objects (wrapper JSObjects) by SpiderMonkey,  SpiderMonkey calls the `JSClass.finalize()` which is basis of each wrapper `JSObject`s. This method refers each `FooBinding::_finalize()`.

In this function, the pointer of Rusty DOM Object that is contained in the wrapper JSObject is unwrapped, it cast to Rust owned pointer, and we assign its owned pointer  to the empty local variable of `FooBinding::_finalize()`. Thus we can destruct the Rusty Object after we left from it.


## Interact with Exact GC’s rooting
For supporting SpiderMonkey’s exact GC rooting, we introduce [some types](https://github.com/mozilla/servo/wiki/Using-DOM-types):

- `JS<T>` is used for the DOM typed field in a DOM type structure. GC can trace them recursively while enclosing DOM object (maybe root) is alive.
- `Temporary<T>` is used as a return value of functions returning DOM type. They are rooted while they are alive. But a retun value gets moved around. It’s breakable for the LIFO ordering constraint. Thus we need introduce `Root<T>`.
- `Root<T>` contains the pointer to `JSObject` which the represented DOM type has. SpiderMonkey's conservative stack scanner scans its pointer and mark a pointed `JSObject` as GC root.
- `JSRef` is just a reference to the value rooted by `Root<T>`.
- `RootCollection` is used for dynamic checking about rooting satisfies LIFO ordering, because SpiderMonkey GC requres LIFO order (See also: [Exact Stack Rooting - Storing a GCPointer on the CStack](https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals/GC/Exact_Stack_Rooting)).
