const UI = {
  createElement: config => {
    if (!config) return document.createElement("div");
    const elementType = config.element || "div";
    const element = document.createElement(elementType);

    Object.keys(config).forEach(property => {
      const value = config[property];
      switch (property.toLowerCase()) {
        case "id":
        case "src":
        case "style":
        case "placeholder":
        case "title":
          element.setAttribute(property, value);
          return;
        case "classname":
          element.setAttribute("class", value);
          return;
        case "text":
          element.innerText = value;
          return;
        case "html":
          element.innerHTML = value;
          return;
        case "onclick":
          element.onclick = value.bind(element);
          return;
        case "onchange":
          element.onchange = value.bind(element);
          return;
        case "onkeydown":
          element.onkeydown = value.bind(element);
          return;
        case "type":
          if (elementType === "input") element.setAttribute("type", value);
          return;
        case "children":
          if (value instanceof Array) {
            value.forEach(child =>
              element.appendChild(
                child instanceof Element ? child : UI.createElement(child)
              )
            );
          } else {
            element.appendChild(
              value instanceof Element ? value : UI.createElement(value)
            );
          }
          return;
        case "disabled":
          if (value) element.setAttribute("disabled", true);
          return;
      }
    });
    return element;
  },
  getElement: id => {
    return document.getElementById(id);
  },
  getRoot: () => {
    return document.getElementsByTagName("body")[0];
  }
};
