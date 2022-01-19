# IDL Tests (idlharness.js)

## Introduction ##

`idlharness.js` generates tests for Web IDL fragments, using the
[JavaScript Tests (`testharness.js`)](testharness.md) infrastructure. You typically want to use
`.any.js` or `.window.js` for this to avoid having to write unnessary boilerplate.

## Adding IDL fragments

Web IDL is automatically scraped from specifications and added to the `/interfaces/` directory. See
the [README](https://github.com/web-platform-tests/wpt/blob/master/interfaces/README.md) there for
details.

## Testing IDL fragments

For example, the Fetch API's IDL is tested in
[`/fetch/api/idlharness.any.js`](https://github.com/web-platform-tests/wpt/blob/master/fetch/api/idlharness.any.js):
```js
// META: global=window,worker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

idl_test(
  ['fetch'],
  ['referrer-policy', 'html', 'dom'],
  idl_array => {
    idl_array.add_objects({
      Headers: ["new Headers()"],
      Request: ["new Request('about:blank')"],
      Response: ["new Response()"],
    });
    if (self.GLOBAL.isWindow()) {
      idl_array.add_objects({ Window: ['window'] });
    } else if (self.GLOBAL.isWorker()) {
      idl_array.add_objects({ WorkerGlobalScope: ['self'] });
    }
  }
);
```
Note how it includes `/resources/WebIDLParser.js` and `/resources/idlharness.js` in addition to
`testharness.js` and `testharnessreport.js` (automatically included due to usage of `.any.js`).
These are needed to make the `idl_test` function work.

The `idl_test` function takes three arguments:

* _srcs_: a list of specifications whose IDL you want to test. The names here need to match the filenames (excluding the extension) in `/interfaces/`.
* _deps_: a list of specifications the IDL listed in _srcs_ depends upon. Be careful to list them in the order that the dependencies are revealed.
* _setup_func_: a function or async function that takes care of creating the various objects that you want to test.

## Methods of `IdlArray` ##

`IdlArray` objects can be obtained through the _setup_func_ argument of `idl_test`. Anything not
documented in this section should be considered an implementation detail, and outside callers should
not use it.

### `add_objects(dict)`

_dict_ should be an object whose keys are the names of interfaces or exceptions, and whose values
are arrays of strings.  When an interface or exception is tested, every string registered for it
with `add_objects()` will be evaluated, and tests will be run on the result to verify that it
correctly implements that interface or exception.  This is the only way to test anything about
`[LegacyNoInterfaceObject]` interfaces, and there are many tests that can't be run on any interface
without an object to fiddle with.

The interface has to be the *primary* interface of all the objects provided.  For example, don't
pass `{Node: ["document"]}`, but rather `{Document: ["document"]}`.  Assuming the `Document`
interface was declared to inherit from `Node`, this will automatically test that document implements
the `Node` interface too.

Warning: methods will be called on any provided objects, in a manner that WebIDL requires be safe.
For instance, if a method has mandatory arguments, the test suite will try calling it with too few
arguments to see if it throws an exception. If an implementation incorrectly runs the function
instead of throwing, this might have side effects, possibly even preventing the test suite from
running correctly.

### `prevent_multiple_testing(name)`

This is a niche method for use in case you're testing many objects that implement the same
interfaces, and don't want to retest the same interfaces every single time. For instance, HTML
defines many interfaces that all inherit from `HTMLElement`, so the HTML test suite has something
like

```js
.add_objects({
  HTMLHtmlElement: ['document.documentElement'],
  HTMLHeadElement: ['document.head'],
  HTMLBodyElement: ['document.body'],
  ...
})
```

and so on for dozens of element types.  This would mean that it would retest that each and every one
of those elements implements `HTMLElement`, `Element`, and `Node`, which would be thousands of
basically redundant tests. The test suite therefore calls `prevent_multiple_testing("HTMLElement")`.
This means that once one object has been tested to implement `HTMLElement` and its ancestors, no
other object will be.  Thus in the example code above, the harness would test that
`document.documentElement` correctly implements `HTMLHtmlElement`, `HTMLElement`, `Element`, and
`Node`; but `document.head` would only be tested for `HTMLHeadElement`, and so on for further
objects.
