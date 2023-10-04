[
  {
    "property": "origin",
    "get": "null",
    "set": null
  },
  {
    "property": "protocol",
    "get": "non-special:",
    "set": "super-special",
    "setget": "super-special:"
  },
  {
    "property": "username"
  },
  {
    "property": "password"
  },
  {
    "property": "host",
  },
  {
    "property": "hostname",
  },
  {
    "property": "port",
    "set": "8000"
  },
  {
    "property": "pathname",
    "get": "opaque",
    "setget": "opaque"
  },
  {
    "property": "search",
    "setget": "?string"
  },
  {
    "property": "hash",
    "setget": "#string"
  }
].forEach(({ property, get = "", set = "string", setget = get }) => {
  ["a", "area"].forEach(name => {
    test(() => {
      const link = document.createElement(name);
      link.href = "non-special:opaque";
      assert_equals(link[property], get);
    }, `<${name} href="non-special:opaque">.${property} getter`);

    if (set !== null) {
      test(() => {
        const link = document.createElement(name);
        link.href = "non-special:opaque";
        link[property] = set;
        assert_equals(link[property], setget);
      }, `<${name} href="non-special:opaque">.${property} setter`);
    }
  });
});
