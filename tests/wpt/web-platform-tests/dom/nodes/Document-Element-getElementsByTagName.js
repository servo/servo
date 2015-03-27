function test_getElementsByTagName(context, element) {
  // TODO: getElementsByTagName("*")
  test(function() {
    assert_false(context.getElementsByTagName("html") instanceof NodeList,
                 "Should not return a NodeList")
    assert_true(context.getElementsByTagName("html") instanceof HTMLCollection,
                "Should return an HTMLCollection")
  }, "Interfaces")

  test(function() {
    var firstCollection = context.getElementsByTagName("html"),
        secondCollection = context.getElementsByTagName("html")
    assert_true(firstCollection !== secondCollection ||
                firstCollection === secondCollection)
  }, "Caching is allowed")

  test(function() {
    var l = context.getElementsByTagName("nosuchtag")
    l[5] = "foopy"
    assert_equals(l[5], undefined)
    assert_equals(l.item(5), null)
  }, "Shouldn't be able to set unsigned properties on a HTMLCollection (non-strict mode)")

  test(function() {
    var l = context.getElementsByTagName("nosuchtag")
    assert_throws(new TypeError(), function() {
      "use strict";
      l[5] = "foopy"
    })
    assert_equals(l[5], undefined)
    assert_equals(l.item(5), null)
  }, "Shouldn't be able to set unsigned properties on a HTMLCollection (strict mode)")

  test(function() {
    var l = context.getElementsByTagName("nosuchtag")
    var fn = l.item;
    assert_equals(fn, HTMLCollection.prototype.item);
    l.item = "pass"
    assert_equals(l.item, "pass")
    assert_equals(HTMLCollection.prototype.item, fn);
  }, "Should be able to set expando shadowing a proto prop (item)")

  test(function() {
    var l = context.getElementsByTagName("nosuchtag")
    var fn = l.namedItem;
    assert_equals(fn, HTMLCollection.prototype.namedItem);
    l.namedItem = "pass"
    assert_equals(l.namedItem, "pass")
    assert_equals(HTMLCollection.prototype.namedItem, fn);
  }, "Should be able to set expando shadowing a proto prop (namedItem)")

  test(function() {
    var t = element.appendChild(document.createElement("pre"));
    t.id = "x";
    this.add_cleanup(function() {element.removeChild(t)});

    var list = context.getElementsByTagName('pre');
    var pre = list[0];
    assert_equals(pre.id, "x");
    assert_equals(list['x'], pre);

    assert_true('x' in list, "'x' in list");
    assert_true(list.hasOwnProperty('x'), "list.hasOwnProperty('x')");

    assert_array_equals(Object.getOwnPropertyNames(list).sort(), ["0", "x"]);

    var desc = Object.getOwnPropertyDescriptor(list, '0');
    assert_equals(typeof desc, "object", "descriptor should be an object");
    assert_true(desc.enumerable, "desc.enumerable");
    assert_true(desc.configurable, "desc.configurable");

    desc = Object.getOwnPropertyDescriptor(list, 'x');
    assert_equals(typeof desc, "object", "descriptor should be an object");
    assert_false(desc.enumerable, "desc.enumerable");
    assert_true(desc.configurable, "desc.configurable");
  }, "hasOwnProperty, getOwnPropertyDescriptor, getOwnPropertyNames")

  test(function() {
    assert_equals(document.createElementNS("http://www.w3.org/1999/xhtml", "i").localName, "i") // Sanity
    var t = element.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "I"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_equals(t.localName, "I")
    assert_equals(t.tagName, "I")
    assert_equals(context.getElementsByTagName("I").length, 0)
    assert_equals(context.getElementsByTagName("i").length, 0)
  }, "HTML element with uppercase tagName never matches in HTML Documents")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "st"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("st"), [t])
    assert_array_equals(context.getElementsByTagName("ST"), [])
  }, "Element in non-HTML namespace, no prefix, lowercase name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "ST"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("ST"), [t])
    assert_array_equals(context.getElementsByTagName("st"), [])
  }, "Element in non-HTML namespace, no prefix, uppercase name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "te:st"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("st"), [t])
    assert_array_equals(context.getElementsByTagName("ST"), [])
  }, "Element in non-HTML namespace, prefix, lowercase name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "te:ST"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("ST"), [t])
    assert_array_equals(context.getElementsByTagName("st"), [])
    assert_array_equals(context.getElementsByTagName("te:st"), [])
    assert_array_equals(context.getElementsByTagName("te:ST"), [])
  }, "Element in non-HTML namespace, prefix, uppercase name")

  test(function() {
    var t = element.appendChild(document.createElement("aÇ"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_equals(t.localName, "aÇ")
    assert_array_equals(context.getElementsByTagName("AÇ"), [t], "All uppercase input")
    assert_array_equals(context.getElementsByTagName("aÇ"), [t], "Ascii lowercase input")
    assert_array_equals(context.getElementsByTagName("aç"), [], "All lowercase input")
  }, "Element in HTML namespace, no prefix, non-ascii characters in name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "AÇ"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("AÇ"), [t])
    assert_array_equals(context.getElementsByTagName("aÇ"), [])
    assert_array_equals(context.getElementsByTagName("aç"), [])
  }, "Element in non-HTML namespace, non-ascii characters in name")

  test(function() {
    var t = element.appendChild(document.createElementNS("http://www.w3.org/1999/xhtml", "test:aÇ"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("AÇ"), [t], "All uppercase input")
    assert_array_equals(context.getElementsByTagName("aÇ"), [t], "Ascii lowercase input")
    assert_array_equals(context.getElementsByTagName("aç"), [], "All lowercase input")
  }, "Element in HTML namespace, prefix, non-ascii characters in name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "test:AÇ"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("AÇ"), [t], "All uppercase input")
    assert_array_equals(context.getElementsByTagName("aÇ"), [], "Ascii lowercase input")
    assert_array_equals(context.getElementsByTagName("aç"), [], "All lowercase input")
  }, "Element in non-HTML namespace, prefix, non-ascii characters in name")

  test(function() {
    var actual = context.getElementsByTagName("*");
    var expected = [];
    var get_elements = function(node) {
      for (var i = 0; i < node.childNodes.length; i++) {
        var child = node.childNodes[i];
        if (child.nodeType === child.ELEMENT_NODE) {
          expected.push(child);
          get_elements(child);
        }
      }
    }
    get_elements(context);
    assert_array_equals(actual, expected);
  }, "getElementsByTagName('*')")
}
