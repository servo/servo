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
    var t1 = element.appendChild(document.createElement("pre"));
    t1.id = "x";
    var t2 = element.appendChild(document.createElement("pre"));
    t2.setAttribute("name", "y");
    var t3 = element.appendChild(document.createElementNS("", "pre"));
    t3.setAttribute("id", "z");
    var t4 = element.appendChild(document.createElementNS("", "pre"));
    t4.setAttribute("name", "w");
    this.add_cleanup(function() {
      element.removeChild(t1)
      element.removeChild(t2)
      element.removeChild(t3)
      element.removeChild(t4)
    });

    var list = context.getElementsByTagName('pre');
    var pre = list[0];
    assert_equals(pre.id, "x");

    var exposedNames = { 'x': 0, 'y': 1, 'z': 2 };
    for (var exposedName in exposedNames) {
      assert_equals(list[exposedName], list[exposedNames[exposedName]]);
      assert_equals(list[exposedName], list.namedItem(exposedName));
      assert_true(exposedName in list, "'" + exposedName + "' in list");
      assert_true(list.hasOwnProperty(exposedName),
                  "list.hasOwnProperty('" + exposedName + "')");
    }

    var unexposedNames = ["w"];
    for (var unexposedName of unexposedNames) {
      assert_false(unexposedName in list);
      assert_false(list.hasOwnProperty(unexposedName));
      assert_equals(list[unexposedName], undefined);
      assert_equals(list.namedItem(unexposedName), null);
    }

    assert_array_equals(Object.getOwnPropertyNames(list).sort(),
                        ["0", "1", "2", "3", "x", "y", "z"]);

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
    assert_array_equals(context.getElementsByTagName("st"), [])
    assert_array_equals(context.getElementsByTagName("ST"), [])
    assert_array_equals(context.getElementsByTagName("te:st"), [t])
    assert_array_equals(context.getElementsByTagName("te:ST"), [])
  }, "Element in non-HTML namespace, prefix, lowercase name")

  test(function() {
    var t = element.appendChild(document.createElementNS("test", "te:ST"))
    this.add_cleanup(function() {element.removeChild(t)})
    assert_array_equals(context.getElementsByTagName("st"), [])
    assert_array_equals(context.getElementsByTagName("ST"), [])
    assert_array_equals(context.getElementsByTagName("te:st"), [])
    assert_array_equals(context.getElementsByTagName("te:ST"), [t])
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
