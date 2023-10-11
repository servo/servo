'use strict';

function navigateFocusForward() {
  // TAB = '\ue004'
  return test_driver.send_keys(document.body, "\ue004");
}

async function navigateFocusBackward() {
  return new test_driver.Actions()
    .keyDown('\uE050')
    .keyDown('\uE004')
    .keyUp('\uE004')
    .keyUp('\uE050')
    .send();
}

// If shadow root is open, can find element using element path
// If shadow root is open, can find the shadowRoot from the element

function innermostActiveElement(element) {
  element = element || document.activeElement;
  if (isIFrameElement(element)) {
    if (element.contentDocument.activeElement)
      return innermostActiveElement(element.contentDocument.activeElement);
    return element;
  }
  if (isShadowHost(element)) {
    let shadowRoot = element.shadowRoot;
    if (shadowRoot) {
      if (shadowRoot.activeElement)
        return innermostActiveElement(shadowRoot.activeElement);
    }
  }
  return element;
}

function isInnermostActiveElement(path) {
  const element = getNodeInComposedTree(path);
  if (!element)
    return false;
  return element === innermostActiveElement();
}

async function shouldNavigateFocus(fromElement, direction) {
  if (!fromElement)
    return false;

  fromElement.focus();
  if (fromElement !== innermostActiveElement())
    return false;

  if (direction == 'forward')
    await navigateFocusForward();
  else
    await navigateFocusBackward();

  return true;
}

async function assert_focus_navigation_element(fromPath, toPath, direction) {
  const fromElement = getNodeInComposedTree(fromPath);
  const result = await shouldNavigateFocus(fromElement, direction);
  assert_true(result, 'Failed to focus ' + fromPath);

  const message =
    `Focus should move ${direction} from ${fromPath} to ${toPath}`;
  const toElement = getNodeInComposedTree(toPath);
  assert_equals(innermostActiveElement(), toElement, message);
}

async function assert_focus_navigation_elements(elements, direction) {
  assert_true(
    elements.length >= 2,
    'length of elements should be greater than or equal to 2.');
  for (var i = 0; i + 1 < elements.length; ++i)
    await assert_focus_navigation_element(elements[i], elements[i + 1], direction);

}

async function assert_focus_navigation_forward(elements) {
  return assert_focus_navigation_elements(elements, 'forward');
}

async function assert_focus_navigation_backward(elements) {
  return assert_focus_navigation_elements(elements, 'backward');
}

async function assert_focus_navigation_bidirectional(elements) {
  await assert_focus_navigation_forward(elements);
  elements.reverse();
  await assert_focus_navigation_backward(elements);
}


// If shadow root is closed, need to pass shadowRoot and element to find
// innermost active element

function isShadowHostOfRoot(shadowRoot, node) {
  return shadowRoot && shadowRoot.host.isEqualNode(node);
}

function innermostActiveElementWithShadowRoot(shadowRoot, element) {
  element = element || document.activeElement;
  if (isIFrameElement(element)) {
    if (element.contentDocument.activeElement)
      return innermostActiveElementWithShadowRoot(shadowRoot, element.contentDocument.activeElement);
    return element;
  }
  if (isShadowHostOfRoot(shadowRoot, element)) {
    if (shadowRoot.activeElement)
      return innermostActiveElementWithShadowRoot(shadowRoot, shadowRoot.activeElement);
  }
  return element;
}

async function shouldNavigateFocusWithShadowRoot(from, direction) {
  const [fromElement, shadowRoot] = from;
  if (!fromElement)
    return false;

  fromElement.focus();
  if (fromElement !== innermostActiveElementWithShadowRoot(shadowRoot))
    return false;

  if (direction == 'forward')
    await navigateFocusForward();
  else
    await navigateFocusBackward();

  return true;
}

async function assert_focus_navigation_element_with_shadow_root(from, to, direction) {
  const result = await shouldNavigateFocusWithShadowRoot(from, direction);
  const [fromElement] = from;
  const [toElement, toShadowRoot] = to;
  assert_true(result, 'Failed to focus ' + fromElement.id);
  const message =
    `Focus should move ${direction} from ${fromElement.id} to ${toElement.id}`;
  assert_equals(innermostActiveElementWithShadowRoot(toShadowRoot), toElement, message);
}

async function assert_focus_navigation_elements_with_shadow_root(elements, direction) {
  assert_true(
    elements.length >= 2,
    'length of elements should be greater than or equal to 2.');
  for (var i = 0; i + 1 < elements.length; ++i)
    await assert_focus_navigation_element_with_shadow_root(elements[i], elements[i + 1], direction);
}

async function assert_focus_navigation_forward_with_shadow_root(elements) {
  return assert_focus_navigation_elements_with_shadow_root(elements, 'forward');
}

async function assert_focus_navigation_backward_with_shadow_root(elements) {
  return assert_focus_navigation_elements_with_shadow_root(elements, 'backward');
}

async function assert_focus_navigation_bidirectional_with_shadow_root(elements) {
  await assert_focus_navigation_forward_with_shadow_root(elements);
  elements.reverse();
  await assert_focus_navigation_backward_with_shadow_root(elements);
}

