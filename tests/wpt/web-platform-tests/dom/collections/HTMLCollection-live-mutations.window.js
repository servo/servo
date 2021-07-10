function testHTMLCollection(name, hooks) {
  test(() => {
    const nodes = {
      root: document.createElement("div"),
      div1: document.createElement("div"),
      div2: document.createElement("div"),
      p: document.createElement("p")
    };

    nodes.div1.id = "div1";
    nodes.div2.id = "div2";

    const list = nodes.root.getElementsByTagName("div");

    hooks.initial(list, nodes);

    nodes.root.appendChild(nodes.div1);
    nodes.root.appendChild(nodes.p);
    nodes.root.appendChild(nodes.div2);

    hooks.afterInsertion(list, nodes);

    nodes.root.removeChild(nodes.div1);

    hooks.afterRemoval(list, nodes);
  }, `HTMLCollection live mutations: ${name}`);
}

testHTMLCollection("HTMLCollection.length", {
  initial(list) {
    assert_equals(list.length, 0);
  },
  afterInsertion(list) {
    assert_equals(list.length, 2);
  },
  afterRemoval(list) {
    assert_equals(list.length, 1);
  }
});

testHTMLCollection("HTMLCollection.item(index)", {
  initial(list) {
    assert_equals(list.item(0), null);
  },
  afterInsertion(list, nodes) {
    assert_equals(list.item(0), nodes.div1);
    assert_equals(list.item(1), nodes.div2);
  },
  afterRemoval(list, nodes) {
    assert_equals(list.item(0), nodes.div2);
  }
});

testHTMLCollection("HTMLCollection[index]", {
  initial(list) {
    assert_equals(list[0], undefined);
  },
  afterInsertion(list, nodes) {
    assert_equals(list[0], nodes.div1);
    assert_equals(list[1], nodes.div2);
  },
  afterRemoval(list, nodes) {
    assert_equals(list[0], nodes.div2);
  }
});

testHTMLCollection("HTMLCollection.namedItem(index)", {
  initial(list) {
    assert_equals(list.namedItem("div1"), null);
    assert_equals(list.namedItem("div2"), null);
  },
  afterInsertion(list, nodes) {
    assert_equals(list.namedItem("div1"), nodes.div1);
    assert_equals(list.namedItem("div2"), nodes.div2);
  },
  afterRemoval(list, nodes) {
    assert_equals(list.namedItem("div1"), null);
    assert_equals(list.namedItem("div2"), nodes.div2);
  }
});

testHTMLCollection("HTMLCollection ownPropertyNames", {
  initial(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), []);
  },
  afterInsertion(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), ["0", "1", "div1", "div2"]);
  },
  afterRemoval(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), ["0", "div2"]);
  }
});

