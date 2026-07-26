class CanvasWidget extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({mode: 'open'});
    this.shadowRoot.innerHTML = `
      <style>
        #child {
          width: 100px;
          height: 200px;
          background: blue;
        }
        #sameOrigin {
          position: absolute;
          left: 0px;
          top: 0px;
        }
        #crossOrigin {
          position: absolute;
          left: 0px;
          top: 100px;
        }
      </style>
      <canvas id=canvas width="100" height="200" layoutsubtree>
        <div id=child>
        </div>
      </canvas>
    `;

    this.canvas = this.shadowRoot.getElementById('canvas');
    this.ctx = this.canvas.getContext('2d');
  }

  async connectedCallback() {
    const loadImage = (id, url) => new Promise((resolve, reject) => {
      const img = new Image();
      img.id = id;
      img.src = url;
      img.onload = () => resolve(img);
      img.onerror = () => reject(new Error(`Failed to load image: ${url}`));
    });

    const sameOriginImg =
        await loadImage('sameOrigin', '/images/green-100x100.png');
    const crossOriginImg = await loadImage(
        'crossOrigin',
        'https://{{hosts[alt][www]}}:{{ports[https][0]}}/images/red-100x100.png');

    let child = this.shadowRoot.getElementById('child');
    child.appendChild(sameOriginImg);
    child.appendChild(crossOriginImg);

    this.setupEventListeners();
    this.canvas.requestPaint();
  }

  setupEventListeners() {
    this.canvas.addEventListener('paint', (event) => {
      let child = this.shadowRoot.getElementById('child');
      this.ctx.drawElementImage(child, 0, 0);

      // Fetch all pixel data once to avoid multiple slow readbacks.
      const imgData =
          this.ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);

      const same_index = (50 * imgData.width + 50) * 4;
      const same_array = imgData.data.slice(same_index, same_index + 4);
      const cross_index = (150 * imgData.width + 50) * 4;
      const cross_array = imgData.data.slice(cross_index, cross_index + 4);

      const customEvent = new CustomEvent('paint-received', {
        detail: {same: same_array, cross: cross_array},
        bubbles: true,
        composed: true
      });

      this.dispatchEvent(customEvent);
    });
  }
}

customElements.define('canvas-widget', CanvasWidget);
