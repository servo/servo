var STRINGS = {
  unescapedHTML: "<html>This has ', \", >, <, &amp; & characters!</html>",
  escapedHTML: "&lt;html&gt;This has &#39;, &quot;, &gt;, &lt;, &amp;amp; &amp; characters!&lt;/html&gt;",
  unescapedText: "This has ', \", >, <, & & characters!",
};

var URLS = {
  safe: "https://example.test/",
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
