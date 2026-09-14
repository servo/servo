// META: title=Foster parenting: script moves the table between two insertions for one token

// Spec: https://html.spec.whatwg.org/#appropriate-place-for-inserting-a-node
//       https://html.spec.whatwg.org/#reconstruct-the-active-formatting-elements
//       https://html.spec.whatwg.org/#adoption-agency-algorithm
//
// A single "nobr" start tag foster parents twice: once when reconstructing the active
// formatting elements, and again after the adoption agency algorithm finds no furthest
// block and pops the stack of open elements back to the <tr>. The <script> holding the
// table is therefore the foster parent, and inserting into it runs it, so the script can
// move or remove the table in between. The appropriate place for inserting a node is
// computed per insertion, so the second one has to account for that.

const variants = [
  {
    name: "moved to a div",
    container: `<div id="destination"></div>`,
    move: `destination.append(table)`,
    // The table's new parent is not a template, so it keeps the table as reference child.
    parent: "destination",
    beforeTable: true,
  },
  {
    name: "moved to a template element",
    container: `<template id="destination"></template>`,
    move: `destination.append(table)`,
    // A template's insertion target is null, so this redirects into its template contents
    // and discards the reference child.
    parent: "template contents",
  },
  {
    name: "removed from the tree",
    container: ``,
    move: `table.remove()`,
    // With no parent to use, the foster parent is the element above the table on the
    // stack of open elements.
    parent: "body",
  },
];

const markupFor = variant => `${variant.container}<table><nobr><b><tr><script>
window.moveTable = () => {
  const table = document.querySelector("table");
  window.tableParentWhenRun = table.parentNode.id;
  ${variant.move};
};
const fosterParent = document.createElement("script");
fosterParent.id = "foster-parent";
// An unknown type makes "prepare the script element" return before it sets "already
// started", so correcting the type below leaves the script still runnable.
fosterParent.type = "unknown/type";
fosterParent.append("moveTable()");
document.head.append(fosterParent);
fosterParent.append(document.querySelector("table"));
fosterParent.type = "";
</script><nobr></table>`;

function parse(markup) {
  const iframe = document.createElement("iframe");
  iframe.srcdoc = markup;
  return new Promise(resolve => {
    iframe.onload = () => resolve(iframe);
    document.body.append(iframe);
  });
}

// The document tree plus any template contents, which are a separate tree.
function roots(doc) {
  return [doc, ...[...doc.querySelectorAll("template")].map(template => template.content)];
}

// Every parent/child/sibling link that disagrees. Inserting before a reference child
// that has moved away shows up here as a node listed among a parent's children whose
// parentNode is something else.
function brokenLinks(doc) {
  const name = node => `<${node.nodeName.toLowerCase()}${node.id ? "#" + node.id : ""}>`;
  const problems = [];
  const seen = new Set();
  const walk = node => {
    if (seen.has(node)) {
      problems.push(`${name(node)} has two parents`);
      return;
    }
    seen.add(node);
    const children = [];
    for (let child = node.firstChild; child && children.length < 100; child = child.nextSibling)
      children.push(child);
    children.forEach((child, i) => {
      if (child.parentNode !== node)
        problems.push(`${name(node)} lists ${name(child)}, whose parentNode is elsewhere`);
      if (child.previousSibling !== (children[i - 1] ?? null))
        problems.push(`${name(node)} has a broken previousSibling at ${name(child)}`);
      walk(child);
    });
    if (node.lastChild !== (children.at(-1) ?? null))
      problems.push(`${name(node)}'s lastChild is not the end of its child list`);
  };
  roots(doc).forEach(walk);
  return problems;
}

// The second foster-parented element is the only <b> with a <nobr> child, and it is
// reachable only if it was inserted at all.
function secondInsertion(doc) {
  for (const root of roots(doc)) {
    const b = [...root.querySelectorAll("b")].find(b => b.firstElementChild?.localName === "nobr");
    if (b)
      return b;
  }
  return null;
}

function placement(node) {
  if (!node)
    return "not inserted";
  const parent = node.parentNode;
  if (parent.nodeType === Node.DOCUMENT_FRAGMENT_NODE)
    return "template contents";
  return parent.id || parent.localName;
}

for (const variant of variants) {
  const parsed = parse(markupFor(variant));

  promise_test(async () => {
    const doc = (await parsed).contentDocument;
    assert_equals(brokenLinks(doc).join("; "), "", "the tree is well-formed");
  }, `Table ${variant.name}: the tree stays well-formed`);

  promise_test(async () => {
    const iframe = await parsed;
    const doc = iframe.contentDocument;
    assert_equals(iframe.contentWindow.tableParentWhenRun, "foster-parent",
      "the script ran while the parser was foster parenting into it");
    const inserted = secondInsertion(doc);
    assert_equals(placement(inserted), variant.parent,
      "the second foster-parented element follows the table");
    if (variant.beforeTable)
      assert_equals(inserted.nextSibling, doc.querySelector("table"), "immediately before the table");
  }, `Table ${variant.name}: the second foster-parented element follows it`);
}
