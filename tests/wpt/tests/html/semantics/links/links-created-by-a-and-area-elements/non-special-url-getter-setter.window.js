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
    "get": "test:9001",
    "setget": "string:9001"
  },
  {
    "property": "hostname",
    "get": "test"
  },
  {
    "property": "port",
    "get": "9001",
    "set": "8000"
  },
  {
    "property": "pathname",
    "get": "/",
    "setget": "/string"
  },
  {
    "property": "search",
    "setget": "?string"
  },
  {
    "property": "hash",
    "setget": "#string"
  }
].forEach(({ property, get = "", set = "string", setget = set }) => {
  ["a", "area"].forEach(name => {
    test(() => {
      const link = document.createElement(name);
      link.href = "non-special://test:9001/";
      assert_equals(link[property], get);
    }, `<${name} href="non-special://test:9001/">.${property} getter`);

    if (set !== null) {
      test(() => {
        const link = document.createElement(name);
        link.href = "non-special://test:9001/";
        link[property] = set;
        assert_equals(link[property], setget);
      }, `<${name} href="non-special://test:9001/">.${property} setter`);
    }
  });
});
