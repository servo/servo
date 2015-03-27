/*
 * document.createElement(NS)
 *
 * document.getElementsByTagName(NS)
 *
 * Element.setAttribute(NS)
 *
 * Element.getAttribute(NS)
 * Element.hasAttribute(NS)
 * Element.getElementsByTagName(NS)
 */

var tests = [];
setup(function() {
        var name_inputs = ["abc", "Abc", "ABC", "ä", "Ä"];
        var namespaces = ["http://www.w3.org/1999/xhtml", "http://www.w3.org/2000/svg", "http://FOO"];
        name_inputs.forEach(function(x) {
                              tests.push(["createElement " + x, test_create_element, [x]]);
                              tests.push(["setAttribute " +x, test_set_attribute, [x]]);
                              tests.push(["getAttribute " +x, test_get_attribute, [x]]);
                              tests.push(["getElementsByTagName a:" +x, test_get_elements_tag_name,
                                          [outer_product(namespaces, ["a"], name_inputs),
                                           x]]);
                              tests.push(["getElementsByTagName " +x, test_get_elements_tag_name,
                                          [outer_product(namespaces, [null], name_inputs),
                                           x]]);
                            });
        outer_product(namespaces, name_inputs, name_inputs).forEach(function(x) {
                                                                      tests.push(["createElementNS " + x, test_create_element_ns, x]);
                                                                      tests.push(["setAttributeNS " + x, test_set_attribute_ns, x]);
                                                                      tests.push(["getAttributeNS " + x, test_get_attribute_ns, x]);
                                                                    });
        outer_product([null].concat(namespaces), name_inputs).forEach(function(x) {
                                                                        tests.push(["getElementsByTagNameNS " + x, test_get_elements_tag_name_ns,
                                                                        outer_product(namespaces, name_inputs), x]);
                                                                      });
        name_inputs.forEach(function(x) {
                              tests.push(["createElementNS " + x, test_create_element_ns, [null, null, x]]);
                              tests.push(["setAttributeNS " + x, test_set_attribute_ns, [null, null, x]]);
                              tests.push(["getAttributeNS " + x, test_get_attribute_ns, [null, null, x]]);
                            });

      });
function outer_product() {
  var rv = [];
  function compute_outer_product() {
    var args = Array.prototype.slice.call(arguments);
    var index = args[0];
    if (index < args.length) {
      args[index].forEach(function(x) {
                           compute_outer_product.apply(this, [index+1].concat(args.slice(1, index), x, args.slice(index+1)));
                          });
    } else {
      rv.push(args.slice(1));
    }
  }
  compute_outer_product.apply(this, [1].concat(Array.prototype.slice.call(arguments)));
  return rv;
}

function expected_case(input) {
  //is_html gets set by a global on the page loading the tests
  if (is_html) {
    return ascii_lowercase(input);
  } else {
    return input;
  }
}

function ascii_lowercase(input) {
  return input.replace(/[A-Z]/g, function(x) {
                         return x.toLowerCase();
                       });
}

function test_create_element(name) {
  var node = document.createElement(name);
  assert_equals(node.localName, expected_case(name));
}

function test_create_element_ns(namespace, prefix, local_name) {
  var qualified_name = prefix ? prefix + ":" + local_name : local_name;
  var node = document.createElementNS(namespace, qualified_name);
  assert_equals(node.prefix, prefix, "prefix");
  assert_equals(node.localName, local_name, "localName");
}

function test_set_attribute(name) {
  var node = document.createElement("div");
  node.setAttribute(name, "test");
  assert_equals(node.attributes[0].localName, expected_case(name));
}

function test_set_attribute_ns(namespace, prefix, local_name) {
  var qualified_name = prefix ? prefix + ":" + local_name : local_name;
  var node = document.createElement("div");
  node.setAttributeNS(namespace, qualified_name, "test");
  var attr = node.attributes[0];
  assert_equals(attr.prefix, prefix, "prefix");
  assert_equals(attr.localName, local_name, "localName");
}

function test_get_attribute(name) {
  var node = document.createElement("div");
  node.setAttribute(name, "test");
  var expected_name = expected_case(name);
  assert_equals(node.getAttribute(expected_name), "test");
  if (expected_name != name) {
    assert_equals(node.getAttribute(expected_name), "test");
  } else if (name !== ascii_lowercase(name)) {
    assert_equals(node.getAttribute(ascii_lowercase(name)), null);
  }
}

function test_get_attribute_ns(namespace, prefix, local_name) {
  var qualified_name = prefix ? prefix + ":" + local_name : local_name;
  var node = document.createElement("div");
  node.setAttributeNS(namespace, qualified_name, "test");
  var expected_name = local_name;
  assert_equals(node.getAttributeNS(namespace, expected_name), "test");
  if (local_name !== ascii_lowercase(local_name)) {
    assert_equals(node.getAttributeNS(namespace, ascii_lowercase(local_name)), null);
  }
}

function test_get_elements_tag_name(elements_to_create, search_string) {
  var container = document.createElement("div");
  elements_to_create.forEach(function(x) {
                               var qualified_name = x[1] ? x[1] + ":" + x[2] : x[2];
                               var element = document.createElementNS(x[0], qualified_name);
                               container.appendChild(element);
                             });
  var expected = Array.prototype.filter.call(container.childNodes,
                                            function(node) {
                                              if (is_html && node.namespaceURI === "http://www.w3.org/1999/xhtml") {
                                                return node.localName === expected_case(search_string);
                                              } else {
                                                return node.localName === search_string;
                                              }
                                            });
  document.documentElement.appendChild(container);
  try {
    assert_array_equals(document.getElementsByTagName(search_string), expected);
  } finally {
    document.documentElement.removeChild(container);
  }
}

function test_get_elements_tag_name_ns(elements_to_create, search_input) {
  var search_uri = search_input[0];
  var search_name = search_input[1];
  var container = document.createElement("div");
  elements_to_create.forEach(function(x) {
                               var qualified_name = x[1] ? x[1] + ":" + x[2] : x[2];
                               var element = document.createElementNS(x[0], qualified_name);
                               container.appendChild(element);
                             });
  var expected = Array.prototype.filter.call(container.childNodes,
                                            function(node) {
                                              return node.namespaceURI === search_uri;
                                              return node.localName === search_name;
                                            });
  document.documentElement.appendChild(container);
  try {
    assert_array_equals(document.getElementsByTagNameNS(search_uri, search_name), expected);
  } catch(e) {
    throw e;
  } finally {
    document.documentElement.removeChild(container);
  }
}

function test_func() {
  var func = arguments[0];
  var rest = arguments[1];
  func.apply(this, rest);
}

generate_tests(test_func, tests);
