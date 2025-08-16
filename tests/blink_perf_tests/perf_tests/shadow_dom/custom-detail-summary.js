// We reuse the "backend" of the imperative Shadow DOM Distribution API for a new custom element, <my-detail>/<my-summary>.
//TODO(crbug.com/869308):Emulate other <summary><details> features

class MySummaryElement extends HTMLElement {
  constructor() {
    super();
  }
}
customElements.define("my-summary", MySummaryElement);

customElements.define("my-detail", class extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: "open", slotAssignment: "manual" });
  }
  connectedCallback() {
    const target = this;
    if (!target.shadowRoot.querySelector(':scope > slot')) {
      const slot1 = document.createElement("slot");
      const slot2 = document.createElement("slot");
      const shadowRoot = target.shadowRoot;
      shadowRoot.appendChild(slot1);
      shadowRoot.appendChild(slot2);
      slot1.style.display = "block";
      slot1.style.backgroundColor = "red";
      const observer = new MutationObserver(function(mutations) {
        //Get the first <my-summary> element from <my-detail>'s direct children
        const my_summary = target.querySelector(':scope > my-summary');
        if (my_summary) {
          slot1.assign(my_summary);
        } else {
          slot1.assign();
        }
        slot2.assign(...target.childNodes);
      });
    observer.observe(this, {childList: true});
    }
  }
});
