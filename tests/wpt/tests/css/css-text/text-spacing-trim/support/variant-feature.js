'use strict';

/**
 * Add `style` and `feature` search params to the `style` attribute.
 */
if (window.location.search) {
  const params = new URLSearchParams(window.location.search);
  const styles = params.getAll('style');
  const features = params.getAll('feature');
  for (const feature of features) {
    styles.push(`font-feature-settings: '${feature}'`);
  }
  if (styles.length) {
    document.documentElement.style = styles.join(';');
  }
}
