function removeWhiteSpaceOnlyTextNodes(node)
{
  for (var i = 0; i < node.childNodes.length; i++) {
    var child = node.childNodes[i];
    if (child.nodeType === Node.TEXT_NODE && child.nodeValue.trim().length == 0) {
      node.removeChild(child);
      i--;
    } else if (child.nodeType === Node.ELEMENT_NODE || child.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
      removeWhiteSpaceOnlyTextNodes(child);
    }
  }
  if (node.shadowRoot) {
    removeWhiteSpaceOnlyTextNodes(node.shadowRoot);
  }
}

function createTestTree(node) {

  let ids = {};

  function attachShadowFromTemplate(template) {
    let parent = template.parentNode;
    parent.removeChild(template);
    let shadowRoot;
    if (template.getAttribute('data-mode') === 'v0') {
      // For legacy Shadow DOM
      shadowRoot = parent.createShadowRoot();
    } else {
      shadowRoot = parent.attachShadow({mode: template.getAttribute('data-mode')});
    }
    let id = template.id;
    if (id) {
      shadowRoot.id = id;
      ids[id] = shadowRoot;
    }
    shadowRoot.appendChild(document.importNode(template.content, true));
    return shadowRoot;
  }

  function walk(root) {
    if (root.id) {
      ids[root.id] = root;
    }
    for (let e of Array.from(root.querySelectorAll('[id]'))) {
      ids[e.id] = e;
    }
    for (let e of Array.from(root.querySelectorAll('template'))) {
      walk(attachShadowFromTemplate(e));
    }
  }

  walk(node.cloneNode(true));
  return ids;
}

function dispatchEventWithLog(nodes, target, event) {

  function labelFor(e) {
    return e.id || e.tagName;
  }

  let log = [];
  let attachedNodes = [];
  for (let label in nodes) {
    let startingNode = nodes[label];
    for (let node = startingNode; node; node = node.parentNode) {
      if (attachedNodes.indexOf(node) >= 0)
        continue;
      let id = node.id;
      if (!id)
        continue;
      attachedNodes.push(node);
      node.addEventListener(event.type, (e) => {
        // Record [currentTarget, target, relatedTarget, composedPath()]
        log.push([id,
                  labelFor(e.target),
                  e.relatedTarget ? labelFor(e.relatedTarget) : null,
                  e.composedPath().map((n) => {
                    return labelFor(n);
                  })]);
      });
    }
  }
  target.dispatchEvent(event);
  return log;
}

// TODO(hayato): Merge this into dispatchEventWithLog
function dispatchUAEventWithLog(nodes, target, eventType, callback) {

  function labelFor(e) {
    return e.id || e.tagName;
  }

  let log = [];
  let attachedNodes = [];
  for (let label in nodes) {
    let startingNode = nodes[label];
    for (let node = startingNode; node; node = node.parentNode) {
      if (attachedNodes.indexOf(node) >= 0)
        continue;
      let id = node.id;
      if (!id)
        continue;
      attachedNodes.push(node);
      node.addEventListener(eventType, (e) => {
        // Record [currentTarget, target, relatedTarget, composedPath()]
        log.push([id,
                  labelFor(e.target),
                  e.relatedTarget ? labelFor(e.relatedTarget) : null,
                  e.composedPath().map((n) => {
                    return labelFor(n);
                  })]);
      });
    }
  }
  callback(target);
  return log;
}

// This function assumes that testharness.js is available.
function assert_event_path_equals(actual, expected) {
  assert_equals(actual.length, expected.length);
  for (let i = 0; i < actual.length; ++i) {
    assert_equals(actual[i][0], expected[i][0], 'currentTarget at ' + i + ' should be same');
    assert_equals(actual[i][1], expected[i][1], 'target at ' + i + ' should be same');
    assert_equals(actual[i][2], expected[i][2], 'relatedTarget at ' + i + ' should be same');
    assert_array_equals(actual[i][3], expected[i][3], 'composedPath at ' + i + ' should be same');
  }
}
