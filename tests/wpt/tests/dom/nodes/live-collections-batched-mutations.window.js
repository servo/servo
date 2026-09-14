"use strict";

test(() => {
  const parent = document.createElement("div");
  const { children } = parent;
  const iterator = children[Symbol.iterator]();
  const first = document.createElement("span");
  const second = document.createElement("span");
  second.id = "second";

  parent.append(first, "text", second);

  assert_equals(parent.children, children);
  assert_equals(iterator.next().value, first);
  assert_equals(children.length, 2);
  assert_equals(children[1], second);
  assert_equals(children.item(1), second);
  assert_equals(children.namedItem("second"), second);
  assert_equals(children.second, second);
  assert_array_equals(Object.getOwnPropertyNames(children), ["0", "1", "second"]);

  first.remove();

  assert_array_equals([...children], [second]);
  assert_array_equals(Object.keys(children), ["0"]);
}, "children on Element instances remains live across batched mutations and supported access methods");

test(() => {
  const parent = document.createElement("div");
  const { childNodes } = parent;
  const iterator = childNodes[Symbol.iterator]();
  const first = document.createElement("span");
  const text = document.createTextNode("text");
  const second = document.createElement("span");

  parent.append(first, text, second);

  assert_equals(parent.childNodes, childNodes);
  assert_equals(iterator.next().value, first);
  assert_equals(childNodes.length, 3);
  assert_equals(childNodes[2], second);
  assert_equals(childNodes.item(2), second);
  assert_array_equals(Object.getOwnPropertyNames(childNodes), ["0", "1", "2"]);

  first.remove();

  assert_array_equals([...childNodes], [text, second]);
  assert_array_equals(Object.keys(childNodes), ["0", "1"]);
}, "childNodes on Node instances remains live across batched mutations and supported access methods");
