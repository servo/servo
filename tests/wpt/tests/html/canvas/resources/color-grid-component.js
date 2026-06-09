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
        .red { background-color: rgb(255,0,0) }
        .green { background-color: rgb(0,128,0) }
        .blue { background-color: rgb(32,32,255) }
        .yellow { background-color: rgb(224,224,0) }`;

    shadow.appendChild(style);
    shadow.appendChild(wrapper);
  }
}

customElements.define("color-grid", ColorGrid);
