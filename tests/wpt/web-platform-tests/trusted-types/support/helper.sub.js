var INPUTS = {
  HTML: "Hi, I want to be transformed!",
  SCRIPT: "Hi, I want to be transformed!",
  SCRIPTURL: "http://this.is.a.scripturl.test/",
  URL: "http://hello.i.am.an.url/"
};

var RESULTS = {
  HTML: "Quack, I want to be a duck!",
  SCRIPT: "Meow, I want to be a cat!",
  SCRIPTURL: "http://this.is.a.successful.test/",
  URL: "http://hooray.i.am.successfully.transformed/"
};

function createHTMLJS(html) {
  return html.replace("Hi", "Quack")
      .replace("transformed", "a duck");
}

function createScriptJS(script) {
  return script.replace("Hi", "Meow")
      .replace("transformed", "a cat");
}

function createScriptURLJS(scripturl) {
  return scripturl.replace("scripturl", "successful");
}

function createURLJS(url) {
  return url.replace("hello", "hooray")
      .replace("an.url", "successfully.transformed");
}

function createHTML_policy(win, c) {
  return win.TrustedTypes.createPolicy('SomeHTMLPolicyName' + c, { createHTML: createHTMLJS });
}

function createScript_policy(win, c) {
  return win.TrustedTypes.createPolicy('SomeScriptPolicyName' + c, { createScript: createScriptJS });
}

function createScriptURL_policy(win, c) {
  return win.TrustedTypes.createPolicy('SomeScriptURLPolicyName' + c, { createScriptURL: createScriptURLJS });
}

function createURL_policy(win, c) {
  return win.TrustedTypes.createPolicy('SomeURLPolicyName' + c, { createURL: createURLJS });
}

function assert_element_accepts_trusted_html(win, c, t, tag, attribute, expected) {
  let p = createHTML_policy(win, c);
  let html = p.createHTML(INPUTS.HTML);
  assert_element_accepts_trusted_type(tag, attribute, html, expected);
}

function assert_element_accepts_trusted_script(win, c, t, tag, attribute, expected) {
  let p = createScript_policy(win, c);
  let script = p.createScript(INPUTS.SCRIPT);
  assert_element_accepts_trusted_type(tag, attribute, script, expected);
}

function assert_element_accepts_trusted_script_url(win, c, t, tag, attribute, expected) {
  let p = createScriptURL_policy(win, c);
  let scripturl = p.createScriptURL(INPUTS.SCRIPTURL);
  assert_element_accepts_trusted_type(tag, attribute, scripturl, expected);
}

function assert_element_accepts_trusted_url(win, c, t, tag, attribute, expected) {
  let p = createURL_policy(win, c);
  let url = p.createURL(INPUTS.URL);
  assert_element_accepts_trusted_type(tag, attribute, url, expected);
}

function assert_element_accepts_trusted_type(tag, attribute, value, expected) {
  let elem = document.createElement(tag);
  elem[attribute] = value;
  assert_equals(elem[attribute] + "", expected);
}

function assert_throws_no_trusted_type(tag, attribute, value) {
  let elem = document.createElement(tag);
  assert_throws(new TypeError(), _ => {
    elem[attribute] = value;
  });
}

function assert_element_accepts_trusted_html_explicit_set(win, c, t, tag, attribute, expected) {
  let p = createHTML_policy(win, c);
  let html = p.createHTML(INPUTS.HTML);
  assert_element_accepts_trusted_type_explicit_set(tag, attribute, html, expected);
}

function assert_element_accepts_trusted_script_explicit_set(win, c, t, tag, attribute, expected) {
  let p = createScript_policy(win, c);
  let script = p.createScript(INPUTS.SCRIPT);
  assert_element_accepts_trusted_type_explicit_set(tag, attribute, script, expected);
}

function assert_element_accepts_trusted_script_url_explicit_set(win, c, t, tag, attribute, expected) {
  let p = createScriptURL_policy(win, c);
  let scripturl = p.createScriptURL(INPUTS.SCRIPTURL);
  assert_element_accepts_trusted_type_explicit_set(tag, attribute, scripturl, expected);
}

function assert_element_accepts_trusted_url_explicit_set(win, c, t, tag, attribute, expected) {
  let p = createURL_policy(win, c);
  let url = p.createURL(INPUTS.URL);
  assert_element_accepts_trusted_type_explicit_set(tag, attribute, url, expected);
}

function assert_element_accepts_trusted_type_explicit_set(tag, attribute, value, expected) {
  let elem = document.createElement(tag);
  elem.setAttribute(attribute, value);
  assert_equals(elem[attribute] + "", expected);
}

function assert_throws_no_trusted_type_explicit_set(tag, attribute, value) {
  let elem = document.createElement(tag);
  assert_throws(new TypeError(), _ => {
    elem.setAttribute(attribute, value);
  });
}

function assert_element_accepts_non_trusted_type_explicit_set(tag, attribute, value, expected) {
  let elem = document.createElement(tag);
  elem.setAttribute(attribute, value);
  assert_equals(elem[attribute] + "", expected);
}

let namespace = 'http://www.w3.org/1999/xhtml';
function assert_element_accepts_trusted_html_set_ns(win, c, t, tag, attribute, expected) {
  let p = createHTML_policy(win, c);
  let html = p.createHTML(INPUTS.HTML);
  assert_element_accepts_trusted_type_set_ns(tag, attribute, html, expected);
}

function assert_element_accepts_trusted_script_set_ns(win, c, t, tag, attribute, expected) {
  let p = createScript_policy(win, c);
  let script = p.createScript(INPUTS.SCRIPT);
  assert_element_accepts_trusted_type_set_ns(tag, attribute, script, expected);
}

function assert_element_accepts_trusted_script_url_set_ns(win, c, t, tag, attribute, expected) {
  let p = createScriptURL_policy(win, c);
  let scripturl = p.createScriptURL(INPUTS.SCRIPTURL);
  assert_element_accepts_trusted_type_set_ns(tag, attribute, scripturl, expected);
}

function assert_element_accepts_trusted_url_set_ns(win, c, t, tag, attribute, expected) {
  let p = createURL_policy(win, c);
  let url = p.createURL(INPUTS.URL);
  assert_element_accepts_trusted_type_set_ns(tag, attribute, url, expected);
}

function assert_element_accepts_trusted_type_set_ns(tag, attribute, value, expected) {
  let elem = document.createElement(tag);
  elem.setAttributeNS(namespace, attribute, value);
  let attr_node = elem.getAttributeNodeNS(namespace, attribute);
  assert_equals(attr_node.value + "", expected);
}

function assert_throws_no_trusted_type_set_ns(tag, attribute, value) {
  let elem = document.createElement(tag);
  assert_throws(new TypeError(), _ => {
    elem.setAttributeNS(namespace, attribute, value);
  });
}

function assert_element_accepts_non_trusted_type_set_ns(tag, attribute, value, expected) {
  let elem = document.createElement(tag);
  elem.setAttributeNS(namespace, attribute, value);
  let attr_node = elem.getAttributeNodeNS(namespace, attribute);
  assert_equals(attr_node.value + "", expected);
}
