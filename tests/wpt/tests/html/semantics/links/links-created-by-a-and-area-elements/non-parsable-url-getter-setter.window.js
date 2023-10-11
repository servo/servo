[
  {
    "property": "origin",
    "set": null
  },
  {
    "property": "protocol",
    "get": ":",
    "set": "https"
  },
  {
    "property": "username"
  },
  {
    "property": "password"
  },
  {
    "property": "host"
  },
  {
    "property": "hostname"
  },
  {
    "property": "port",
    "set": "8000"
  },
  {
    "property": "pathname"
  },
  {
    "property": "search"
  },
  {
    "property": "hash"
  }
].forEach(({ property, get = "", set = "string" }) => {
  ["a", "area"].forEach(name => {
    test(() => {
      const link = document.createElement(name);
      link.href = "http://test:test/"; // non-parsable URL
      assert_equals(link[property], get);
    }, `<${name} href="http://test:test/">.${property} getter`);

    if (set !== null) {
      test(() => {
        const link = document.createElement(name);
        link.href = "http://test:test/"; // non-parsable URL
        link[property] = set;
        assert_equals(link[property], get);
        assert_equals(link.href, "http://test:test/");
      }, `<${name} href="http://test:test/">.${property} setter`);
    }
  });
});
