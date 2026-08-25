window.addEventListener('load', setupInitialLetterTestVariants);

function setupInitialLetterTestVariants() {
  const search = window.location.search;
  if (!search) {
    return;
  }
  const params = new URLSearchParams(search);
  const classes = params.getAll('class')
                  .flatMap(value => value.split(','))
                  .filter(value => value);
  let text = params.getAll('text').join('');
  if (!text) {
    if (classes.indexOf('no-descent') >= 0) {
      text = '\xC9\xC9M\xC9';
    } else if (classes.indexOf('no-ascent') >= 0) {
      text = 'ppMp';
    }
  }

  for (const element of document.getElementsByClassName('sample')) {
    element.classList.add(...classes);
    if (text) {
      replaceTextStart(element, text);
    }
  }
}

// Replace the start of the text content of the node.
// Returns the number of characters replaced.
//
// For example,
// `replaceTextStart(element, 'XY')` to the content:
// ```
// <div>ABC</div>
// ```
// produces:
// ```
// <div>XYC</div>
// ```
//
// It has a limited support for separated text nodes and collapsible spaces.
function replaceTextStart(node, text) {
  if (node.nodeType == Node.TEXT_NODE) {
    const content = node.nodeValue;
    const trimmed_content = content.trimStart();
    if (!trimmed_content) {
      return 0;
    }
    const leading_spaces_len = content.length - trimmed_content.length;
    const len = Math.min(text.length, trimmed_content.length);
    node.nodeValue = content.substring(0, leading_spaces_len) +
                     text.substring(0, len) +
                     trimmed_content.substring(len);
    return len;
  }

  if (node.nodeType == Node.ELEMENT_NODE && node.className.indexOf('fake') >= 0) {
    // If this is a fake initial letter, pretend that one character is replaced.
    return 1;
  }

  let total_replaced = 0;
  for (const child of node.childNodes) {
    const replaced = replaceTextStart(child, text);
    if (replaced) {
      total_replaced += replaced;
      text = text.substring(replaced);
      if (!text) {
        return total_replaced;
      }
    }
  }
  return total_replaced;
}
