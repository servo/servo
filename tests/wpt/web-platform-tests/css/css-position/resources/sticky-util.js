/**
 * Builds a generic structure that looks like:
 *
 * <div class="scroller">  // 100x200 viewport
 *   <div class="contents">  // 100x500
 *     <div class="prepadding"></div> // 100x100
 *     <div class="container">  // 300x300 containing block
 *       <div class="filler"></div>  // 100x100
 *       <div class="sticky box"></div>  // 100x100
 *     </div>
 *   </div>
 * </div>
 *
 * If 'inline' is true, the necessary blocks will be marked as inline-block,
 * and the dimensions above are flipped.
 *
 * Returns an 'elements' object which has each of the above elements as an
 * accessible property.
 */
function setupStickyTest(stickyDirection, stickyOffset, inline = false) {
  const elements = {};

  elements.scroller = document.createElement('div');
  elements.scroller.style.position = 'relative';
  elements.scroller.style.width = (inline ? '200px' : '100px');
  elements.scroller.style.height = (inline ? '100px' : '200px');
  elements.scroller.style.overflow = 'scroll';

  elements.contents = document.createElement('div');
  elements.contents.style.height = (inline ? '100%' : '500px');
  elements.contents.style.width = (inline ? '500px' : '100%');

  elements.prepadding = document.createElement('div');
  elements.prepadding.style.height = (inline ? '100%' : '100px');
  elements.prepadding.style.width = (inline ? '100px' : '100%');
  if (inline)
    elements.prepadding.style.display = 'inline-block';

  elements.container = document.createElement('div');
  elements.container.style.height = (inline ? '100%' : '300px');
  elements.container.style.width = (inline ? '300px' : '100%');
  if (inline)
    elements.container.style.display = 'inline-block';

  elements.filler = document.createElement('div');
  elements.filler.style.height = (inline ? '100%' : '100px');
  elements.filler.style.width = (inline ? '100px' : '100%');
  if (inline)
    elements.filler.style.display = 'inline-block';

  elements.sticky = document.createElement('div');
  elements.sticky.style = `${stickyDirection}: ${stickyOffset}px;`;
  elements.sticky.style.position = 'sticky';
  elements.sticky.style.height = (inline ? '100%' : '100px');
  elements.sticky.style.width = (inline ? '100px' : '100%');
  elements.sticky.style.backgroundColor = 'green';
  if (inline)
    elements.sticky.style.display = 'inline-block';

  elements.scroller.appendChild(elements.contents);
  elements.contents.appendChild(elements.prepadding);
  elements.contents.appendChild(elements.container);
  elements.container.appendChild(elements.filler);
  elements.container.appendChild(elements.sticky);

  document.body.appendChild(elements.scroller);

  return elements;
}
