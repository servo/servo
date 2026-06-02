function testNodeList(name, hooks) {
  test(() => {
    const nodes = {
      root: document.createElement("div"),
      div1: document.createElement("div"),
      div2: document.createElement("div"),
      p: document.createElement("p")
    };

    const list = nodes.root.childNodes;

    hooks.initial(list, nodes);

    nodes.root.appendChild(nodes.div1);
    nodes.root.appendChild(nodes.p);
    nodes.root.appendChild(nodes.div2);

    hooks.afterInsertion(list, nodes);

    nodes.root.removeChild(nodes.div1);

    hooks.afterRemoval(list, nodes);
  }, `NodeList live mutations: ${name}`);
}

testNodeList("NodeList.length", {
  initial(list) {
    assert_equals(list.length, 0);
  },
  afterInsertion(list) {
    assert_equals(list.length, 3);
  },
  afterRemoval(list) {
    assert_equals(list.length, 2);
  }
});

testNodeList("NodeList.item(index)", {
  initial(list) {
    assert_equals(list.item(0), null);
  },
  afterInsertion(list, nodes) {
    assert_equals(list.item(0), nodes.div1);
    assert_equals(list.item(1), nodes.p);
    assert_equals(list.item(2), nodes.div2);
  },
  afterRemoval(list, nodes) {
    assert_equals(list.item(0), nodes.p);
    assert_equals(list.item(1), nodes.div2);
  }
});

testNodeList("NodeList[index]", {
  initial(list) {
    assert_equals(list[0], undefined);
  },
  afterInsertion(list, nodes) {
    assert_equals(list[0], nodes.div1);
    assert_equals(list[1], nodes.p);
    assert_equals(list[2], nodes.div2);
  },
  afterRemoval(list, nodes) {
    assert_equals(list[0], nodes.p);
    assert_equals(list[1], nodes.div2);
  }
});

testNodeList("NodeList ownPropertyNames", {
  initial(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), []);
  },
  afterInsertion(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), ["0", "1", "2"]);
  },
  afterRemoval(list) {
    assert_object_equals(Object.getOwnPropertyNames(list), ["0", "1"]);
  }
});

