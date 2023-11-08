'use strict';

/**
 * Add `class` search params to the `class` attribute of the root element.
 */
if (window.location.search) {
  const params = new URLSearchParams(window.location.search);
  if (params.has('class')) {
    const values = params.getAll('class')
                         .flatMap(value => value.split(','));
    document.documentElement.classList.add(...values);
  }
  const lang = params.get('lang');
  if (lang) {
    document.documentElement.lang = lang;
  }
}
