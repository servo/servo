function setInnerHTML(el,content) {
  const fragment = (new DOMParser()).parseFromString(`<pre>${content}</pre>`, 'text/html', {includeShadowRoots: true});
  (el instanceof HTMLTemplateElement ? el.content : el).replaceChildren(...fragment.body.firstChild.childNodes);
}
