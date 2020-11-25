function setInnerHTML(el,content) {
  const fragment = (new DOMParser()).parseFromString(`<pre>${content}</pre>`, 'text/html', {includeShadowRoots: true});
  el.replaceChildren(...fragment.body.firstChild.childNodes);
}
