var STRINGS = {
  unescapedHTML: "<html>This has ', \", >, <, &amp; & characters!</html>",
  escapedHTML: "&lt;html&gt;This has &#39;, &quot;, &gt;, &lt;, &amp;amp; &amp; characters!&lt;/html&gt;",
  unescapedText: "This has ', \", >, <, & & characters!",
};

var URLS = {
  safe: "http://{{host}}:{{ports[http][0]}}/",
  javascript: "javascript:'scripted'",
  external: "custom-handler:whatever",
  sanitized: "about:invalid"
};

function createFrameAndWrite(html) {
  return new Promise((resolve, reject) => {
    var i = document.createElement('iframe');
    i.onload = e => {
      i.contentDocument.open();
      try {
        i.contentDocument.write(html);
      } catch (e) {
        i.remove();
        reject(e);
      }
      i.contentDocument.close();
      resolve(i);
    };
    document.body.appendChild(i);
  });
}

function createFrameAndHref(href) {
  return new Promise((resolve, reject) => {
    var i = document.createElement('iframe');
    i.onload = _ => {
      i.onload = null;
      try {
        i.onload = _ => resolve(i);
        i.contentWindow.location.href = href;
      } catch (ex) {
        i.remove();
        reject(ex);
      }
    };
    document.body.appendChild(i);
  });
}

let trustedHTML = TrustedHTML.escape(STRINGS.unescapedHTML);
function assert_accepts_trusted_html(tag, attribute) {
  let elem = document.createElement(tag);
  elem[attribute] = trustedHTML;
  assert_equals(elem[attribute] + "", STRINGS.unescapedHTML);
}

let trustedURL = TrustedURL.create(URLS.safe);
function assert_accepts_trusted_url(tag, attribute) {
  let elem = document.createElement(tag);
  elem[attribute] = trustedURL;
  assert_equals(elem[attribute] + "", URLS.safe);
}

let trustedScriptURL = TrustedScriptURL.unsafelyCreate(URLS.safe);
function assert_accepts_trusted_script_url(tag, attribute) {
  let elem = document.createElement(tag);
  elem[attribute] = trustedScriptURL;
  assert_equals(elem[attribute] + "", URLS.safe);
}

function assert_throws_no_trusted_type(tag, attribute, value) {
  let elem = document.createElement(tag);
  assert_throws(new TypeError(), _ => {
    elem[attribute] = value;
  });
}
