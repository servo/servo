function assert_implements_container_queries() {
  assert_implements(CSS.supports("container-type:size"), "Basic support for container queries required");
}

function polyfill_declarative_shadow_dom(root) {
  root.querySelectorAll("template[shadowroot]").forEach(template => {
    const mode = template.getAttribute("shadowroot");
    const shadowRoot = template.parentNode.attachShadow({ mode });
    shadowRoot.appendChild(template.content);
    template.remove();
    polyfill_declarative_shadow_dom(shadowRoot);
  });
}
