/**
 * Simulates a backdrop-filter: blur with the given radius by creating copies
 * of the blurred content mirrored at edges.
 * @param {Element} div          The element to display the blurred content in.
 * @param {Element[]} blurNodes  The elements to include in the blur.
 * @param {number} blurRadius    The radius of the backdrop blur.
 */
function simulateBackdropBlur(div, blurNodes, blurRadius) {
  // The amount to assume blur may oversample by beyond the blur radius.
  const BLUR_OVERSAMPLE = 2;

  const isAncestor = function(ancestor, div) {
    while (div && div != ancestor) {
      div = div.offsetParent;
    }
    return !!div;
  }

  const commonAncestor = function(a, b) {
    while (!isAncestor(a, b))
      a = a.offsetParent;
    return a;
  }

  const computeOffset = function(from, to) {
    const ancestor = commonAncestor(from, to);
    let offset = {left: 0, top: 0};
    while (from != ancestor) {
      offset.left += from.offsetLeft;
      offset.top += from.offsetTop;
      from = from.offsetParent;
    }
    while (to != ancestor) {
      offset.left -= to.offsetLeft;
      offset.top -= to.offsetTop;
      to = to.offsetParent;
    }
    return offset;
  }

  // Compute the number of copies needed in each direction.
  const w = div.offsetWidth;
  const h = div.offsetHeight;
  const copiesY = Math.ceil(BLUR_OVERSAMPLE * blurRadius / h);
  const copiesX = Math.ceil(BLUR_OVERSAMPLE * blurRadius / w);

  let clipNode = document.createElement('div');
  clipNode.style.backgroundColor = 'white';
  clipNode.style.position = 'absolute';
  clipNode.style.overflow = 'clip';
  clipNode.style.width = `${w}px`;
  clipNode.style.height = `${h}px`;
  clipNode.style.left = `${div.offsetLeft}px`;
  clipNode.style.top = `${div.offsetTop}px`;
  clipNode.style.opacity = getComputedStyle(div).opacity;
  let filterNode = document.createElement('div');
  clipNode.appendChild(filterNode);
  filterNode.style.width = `${w * (2*copiesX+1)}px`;
  filterNode.style.height = `${h * (2*copiesY+1)}px`;
  filterNode.style.position = 'absolute';
  filterNode.style.lineHeight = '0';
  filterNode.style.top = `${-copiesY * h}px`;
  filterNode.style.left = `${-copiesX * w}px`;
  filterNode.style.filter = `blur(${blurRadius}px)`;

  // Helper to clone everything except the node blurring the content in case
  // the "backdrop" content to blur is an ancestor.
  const cloneExcept = function(div, exclude) {
    let cloned = div.cloneNode(false);
    for (let child of div.children) {
      if (child != exclude) {
        cloned.appendChild(child.cloneNode(true));
      }
    }
    return cloned;
  }

  for (let y = -copiesY; y <= copiesY; y++) {
    for (let x = -copiesX; x <= copiesX; x++) {
      let copy = document.createElement('div');
      copy.style.position = 'relative';
      copy.style.width = `${w}px`;
      copy.style.height = `${h}px`;
      copy.style.display = 'inline-block';
      copy.style.overflow = 'clip';
      copy.setAttribute('data-x', x);
      copy.setAttribute('data-y', y);
      copy.style.transform = `scale(${Math.abs(x)%2 == 0 ? 1 : -1}, ${Math.abs(y)%2 == 0 ? 1 : -1})`
      for (let child of blurNodes) {
        const cloned = cloneExcept(child, div);
        cloned.style.position = 'absolute';
        const offset = computeOffset(child, div);
        cloned.style.top = `${offset.top}px`;
        cloned.style.left = `${offset.left}px`;
        copy.appendChild(cloned);
      }
      filterNode.appendChild(copy);
    }
  }
  div.parentElement.insertBefore(clipNode, div);
}
