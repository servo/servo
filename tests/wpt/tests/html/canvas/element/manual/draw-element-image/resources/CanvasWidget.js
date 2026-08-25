class CanvasWidget extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({mode: 'open'});
    this.shadowRoot.innerHTML = `
      <style>
        #child {
          width: 100px;
          height: 100px;
          background: var(--rect-color, black);
        }
        #canvas {
          background: grey;
        }
      </style>

      <canvas id=canvas width="200" height="200" layoutsubtree>
        <div id=child></div>
      </canvas>
    `;

    this.canvas = this.shadowRoot.getElementById('canvas');
    this.ctx = this.canvas.getContext('2d');
  }

  connectedCallback() {
    this.canvas.addEventListener('paint', (event) => {
      const customEvent = new CustomEvent(
          'paint-received', {detail: {}, bubbles: true, composed: true});

      let child = this.shadowRoot.getElementById('child');
      this.ctx.drawElementImage(child, 20, 30);
      this.dispatchEvent(customEvent);
    });
    this.canvas.requestPaint();
  }
}

customElements.define('canvas-widget', CanvasWidget);
