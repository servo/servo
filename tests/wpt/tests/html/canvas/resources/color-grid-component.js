class ColorGrid extends HTMLElement {
  constructor() {
    super();
  }
  connectedCallback() {
    const shadow = this.attachShadow({ mode: "open" });

    const wrapper = document.createElement("div");
    wrapper.setAttribute("class", "wrapper");
    for (const color of ['red', 'green', 'blue', 'yellow']) {
      const d = document.createElement("div");
      d.setAttribute("class", color);
      wrapper.appendChild(d);
    }

    const style = document.createElement("style");
    style.textContent = `
        :host {
          display: block;
        }
        .wrapper {
          display: grid;
          grid: 1fr 1fr / 1fr 1fr;
          width: 100%;
          height: 100%;
        }
        .wrapper > div {
          width: 100%;
          height: 100%;
        }
        .red { background-color: ${this.constructor.RED} }
        .green { background-color: ${this.constructor.GREEN} }
        .blue { background-color: ${this.constructor.BLUE} }
        .yellow { background-color: ${this.constructor.YELLOW} }`;

    shadow.appendChild(style);
    shadow.appendChild(wrapper);
  }
}

class ColorGridSRGB extends ColorGrid {
  static RED = 'rgb(255 0 0)';
  static GREEN = 'rgb(0 128 0)';
  static BLUE = 'rgb(0 0 255)';
  static YELLOW = 'rgb(224 224 0)';
}

class ColorGridDisplayP3 extends ColorGrid {
  static RED = 'color(display-p3 .83 .1 .05)';
  static GREEN = 'color(display-p3 .22 0.79 0.14)';
  static BLUE = 'color(display-p3 .04 .22 0.92)';
  static YELLOW = 'color(display-p3 0.94 0.95 0.08)';
}

customElements.define("color-grid", ColorGridSRGB);
customElements.define("color-grid-p3", ColorGridDisplayP3);
