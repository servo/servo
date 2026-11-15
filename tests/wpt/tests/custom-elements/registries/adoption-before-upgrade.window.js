// https://html.spec.whatwg.org/multipage/custom-elements.html#concept-upgrade-an-element
// https://html.spec.whatwg.org/multipage/dom.html#html-element-constructors
// https://dom.spec.whatwg.org/#concept-node-adopt

setup({ allow_uncaught_exception: true });

// Inserting appender and element enqueues an upgrade reaction for each. Appender's
// connectedCallback then appends element to newParent, adopting it into another document and
// changing its custom element registry, before element's upgrade reaction runs.
function adoptBeforeUpgrade(t, name, newParent) {
  const appender = document.createElement(`${name}-appender`);
  const element = document.createElement(`${name}-element`);
  customElements.define(`${name}-appender`, class extends HTMLElement {
    connectedCallback() {
      newParent.append(element);
    }
  });
  class UpgradedElement extends HTMLElement {}
  customElements.define(`${name}-element`, UpgradedElement);

  const errors = [];
  const onError = event => errors.push(event.error);
  window.addEventListener('error', onError);
  t.add_cleanup(() => window.removeEventListener('error', onError));

  const fragment = document.createDocumentFragment();
  fragment.append(appender, element);
  document.body.append(fragment);
  t.add_cleanup(() => appender.remove());

  assert_equals(element.ownerDocument, newParent.ownerDocument);
  return { element, UpgradedElement, errors };
}

function assertUpgraded({ element, UpgradedElement, errors }) {
  assert_equals(errors.length, 0, 'reported exceptions');
  assert_true(element instanceof UpgradedElement, 'upgraded');
  assert_true(element.matches(':defined'), ':defined');
}

// The HTMLElement constructor does not find the constructor in element's new registry.
function assertUpgradeFailed({ element, UpgradedElement, errors }) {
  assert_equals(errors.length, 1, 'reported exceptions');
  assert_true(errors[0] instanceof TypeError, 'reported exception is a TypeError');
  assert_false(element instanceof UpgradedElement, 'upgraded');
  assert_false(element.matches(':defined'), ':defined');
}


// Target document has a global registry

test(t => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  t.add_cleanup(() => iframe.remove());
  const contentRegistry = iframe.contentWindow.customElements;
  const result = adoptBeforeUpgrade(t, 'global-registry-target', iframe.contentDocument.body);
  assertUpgradeFailed(result);
  assert_equals(result.element.customElementRegistry, contentRegistry);
}, "Upgrade after adoption with global registry");

test(t => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  t.add_cleanup(() => iframe.remove());
  // Leave the iframe's customElements alone until after the upgrade, in case it is created lazily.
  const result = adoptBeforeUpgrade(t, 'unaccessed-global-registry-target', iframe.contentDocument.body);
  assertUpgradeFailed(result);
  assert_equals(result.element.customElementRegistry, iframe.contentWindow.customElements);
}, "Upgrade after adoption with global registry (customElements of target not accessed before)");


// Target document has a null registry

test(t => {
  const contentDocument = document.implementation.createHTMLDocument();
  const result = adoptBeforeUpgrade(t, 'null-registry-target', contentDocument.body);
  assertUpgraded(result);
  assert_equals(result.element.customElementRegistry, null);
}, "Upgrade after adoption with global registry (null registry target)");

test(t => {
  const template = document.createElement('template');
  const result = adoptBeforeUpgrade(t, 'template-contents-target', template.content);
  assertUpgraded(result);
  assert_equals(result.element.customElementRegistry, null);
}, "Upgrade after adoption with global registry (template contents target)");


// Target document has a scoped registry

test(t => {
  const documentRegistry = new CustomElementRegistry();
  const contentDocument = document.implementation.createHTMLDocument();
  documentRegistry.initialize(contentDocument);
  assert_equals(contentDocument.customElementRegistry, documentRegistry);
  const result = adoptBeforeUpgrade(t, 'scoped-registry-target', contentDocument.body);
  assertUpgraded(result);
  assert_equals(result.element.customElementRegistry, null);
}, "Upgrade after adoption with global registry (scoped registry target)");
