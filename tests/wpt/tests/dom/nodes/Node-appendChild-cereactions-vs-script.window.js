const results = [];
test(() => {
  class Script1 extends HTMLScriptElement {
    constructor() {
      super();
    }
    connectedCallback() {
      results.push("ce connected s1");
    }
  }
  class Script2 extends HTMLScriptElement {
    constructor() {
      super();
    }
    connectedCallback() {
      results.push("ce connected s2");
    }
  }
  customElements.define("script-1", Script1, { extends: "script" });
  customElements.define("script-2", Script2, { extends: "script" });
  const s1 = new Script1();
  s1.textContent = "results.push('s1')";
  const s2 = new Script2();
  s2.textContent = "results.push('s2')";
  document.body.append(s1, s2);
  assert_array_equals(results, ["s1", "s2", "ce connected s1", "ce connected s2"]);
}, "Custom element reactions follow script execution");
