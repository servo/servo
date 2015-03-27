## Introduction ##

`idlharness.js` automatically generates browser tests for WebIDL interfaces, using
the testharness.js framework.  To use, first include the following:

    <script src=/resources/testharness.js></script>
    <script src=/resources/testharnessreport.js></script>
    <script src=/resources/WebIDLParser.js></script>
    <script src=/resources/idlharness.js></script>

Then you'll need some type of IDLs.  Here's some script that can be run on a
spec written in HTML, which will grab all the elements with `class="idl"`,
concatenate them, and replace the body so you can copy-paste:

    var s = "";
    [].forEach.call(document.getElementsByClassName("idl"), function(idl) {
      //https://www.w3.org/Bugs/Public/show_bug.cgi?id=14914
      if (!idl.classList.contains("extract"))
      {
        s += idl.textContent + "\n\n";
      }
    });
    document.body.innerHTML = '<pre></pre>';
    document.body.firstChild.textContent = s;

Once you have that, put it in your script somehow.  The easiest way is to
embed it literally in an HTML file with `<script type=text/plain>` or similar,
so that you don't have to do any escaping.  Another possibility is to put it
in a separate .idl file that's fetched via XHR or similar.  Sample usage:

    var idl_array = new IdlArray();
    idl_array.add_untested_idls("interface Node { readonly attribute DOMString nodeName; };");
    idl_array.add_idls("interface Document : Node { readonly attribute DOMString URL; };");
    idl_array.add_objects({Document: ["document"]});
    idl_array.test();

This tests that `window.Document` exists and meets all the requirements of
WebIDL.  It also tests that window.document (the result of evaluating the
string "document") has URL and nodeName properties that behave as they
should, and otherwise meets WebIDL's requirements for an object whose
primary interface is Document.  It does not test that window.Node exists,
which is what you want if the Node interface is already tested in some other
specification's suite and your specification only extends or refers to it.
Of course, each IDL string can define many different things, and calls to
add_objects() can register many different objects for different interfaces:
this is a very simple example.

## Public methods of IdlArray ##

IdlArray objects can be obtained with `new IdlArray()`.  Anything not
documented in this section should be considered an implementation detail,
and outside callers should not use it.

### `add_idls(idl_string)`
  Parses `idl_string` (throwing on parse error) and adds the results to the
  IdlArray.  All the definitions will be tested when you run test().  If
  some of the definitions refer to other definitions, those must be present
  too.  For instance, if `idl_string` says that `Document` inherits from `Node`,
  the `Node` interface must also have been provided in some call to `add_idls()`
  or `add_untested_idls()`.

### `add_untested_idls(idl_string)`
  Like `add_idls()`, but the definitions will not be tested.  If an untested
  interface is added and then extended with a tested partial interface, the
  members of the partial interface will still be tested.  Also, all the
  members will still be tested for objects added with `add_objects()`, because
  you probably want to test that (for instance) window.document has all the
  properties from `Node`, not just `Document`, even if the `Node` interface itself
  is tested in a different test suite.

### `add_objects(dict)`
  `dict` should be an object whose keys are the names of interfaces or
  exceptions, and whose values are arrays of strings.  When an interface or
  exception is tested, every string registered for it with `add_objects()`
  will be evaluated, and tests will be run on the result to verify that it
  correctly implements that interface or exception.  This is the only way to
  test anything about `[NoInterfaceObject]` interfaces, and there are many
  tests that can't be run on any interface without an object to fiddle with.

  The interface has to be the *primary* interface of all the objects
  provided.  For example, don't pass `{Node: ["document"]}`, but rather
  `{Document: ["document"]}`.  Assuming the `Document` interface was declared to
  inherit from `Node`, this will automatically test that document implements
  the `Node` interface too.

  Warning: methods will be called on any provided objects, in a manner that
  WebIDL requires be safe.  For instance, if a method has mandatory
  arguments, the test suite will try calling it with too few arguments to
  see if it throws an exception.  If an implementation incorrectly runs the
  function instead of throwing, this might have side effects, possibly even
  preventing the test suite from running correctly.

### `prevent_multiple_testing(name)`
  This is a niche method for use in case you're testing many objects that
  implement the same interfaces, and don't want to retest the same
  interfaces every single time.  For instance, HTML defines many interfaces
  that all inherit from `HTMLElement`, so the HTML test suite has something
  like
    `.add_objects({
      HTMLHtmlElement: ['document.documentElement'],
      HTMLHeadElement: ['document.head'],
      HTMLBodyElement: ['document.body'],
      ...
    })`
  and so on for dozens of element types.  This would mean that it would
  retest that each and every one of those elements implements `HTMLElement`,
  `Element`, and `Node`, which would be thousands of basically redundant tests.
  The test suite therefore calls `prevent_multiple_testing("HTMLElement")`.
  This means that once one object has been tested to implement `HTMLElement`
  and its ancestors, no other object will be.  Thus in the example code
  above, the harness would test that `document.documentElement` correctly
  implements `HTMLHtmlElement`, `HTMLElement`, `Element`, and `Node`; but
  `document.head` would only be tested for `HTMLHeadElement`, and so on for
  further objects.

### `test()`
  Run all tests.  This should be called after you've called all other
  methods to add IDLs and objects.
