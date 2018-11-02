test(() => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        win = frame.contentWindow,
        loc = win.location;
  frame.remove();
  assert_equals(win.location, loc);
}, "Window and Location are 1:1 after browsing context removal");

function bcLessLocation() {
  const frame = document.body.appendChild(document.createElement("iframe")),
        win = frame.contentWindow,
        loc = win.location;
  frame.remove();
  return loc;
}

[
  {
    "property": "href",
    "expected": "about:blank",
    "values": ["https://example.com/", "/", "http://test:test/", "test test", "test:test", "chrome:fail"]
  },
  {
    "property": "protocol",
    "expected": "about:",
    "values": ["http", "about", "test"]
  },
  {
    "property": "host",
    "expected": "",
    "values": ["example.com", "test test", "()"]
  },
  {
    "property": "hostname",
    "expected": "",
    "values": ["example.com"]
  },
  {
    "property": "port",
    "expected": "",
    "values": ["80", "", "443", "notaport"]
  },
  {
    "property": "pathname",
    "expected": "blank",
    "values": ["/", "x"]
  },
  {
    "property": "search",
    "expected": "",
    "values": ["test"]
  },
  {
    "property": "hash",
    "expected": "",
    "values": ["test", "#"]
  }
].forEach(testSetup => {
  testSetup.values.forEach(value => {
  	test(() => {
  	  const loc = bcLessLocation();
  	  loc[testSetup.property] = value;
  	  assert_equals(loc[testSetup.property], testSetup.expected);
  	}, "Setting `" + testSetup.property + "` to `" + value + "` of a `Location` object sans browsing context is a no-op");
  });
});

test(() => {
  const loc = bcLessLocation();
  assert_equals(loc.origin, "null");
}, "Getting `origin` of a `Location` object sans browsing context should be \"null\"");

["assign", "replace", "reload"].forEach(method => {
  ["about:blank", "https://example.com/", "/", "http://test:test/", "test test", "test:test", "chrome:fail"].forEach(value => {
    test(() => {
      const loc = bcLessLocation();
      loc[method](value);
      assert_equals(loc.href, "about:blank");
    }, "Invoking `" + method + "` with `" + value + "` on a `Location` object sans browsing context is a no-op");
  });
});

test(() => {
  const loc = bcLessLocation();
  assert_array_equals(loc.ancestorOrigins, []);
}, "Getting `ancestorOrigins` of a `Location` object sans browsing context should be []");
