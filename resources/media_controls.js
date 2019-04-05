console.log('YO');
let controls = document.servoGetMediaControls("@@@id@@@");

let style = document.createElement("style");
style.textContent = `#controls {
   -servo-top-layer: top;
   display: block;
   position: fixed;
   left: 0px;
   bottom: 0px;
   height: 30px;
   width: 100%;
   background: blue;
}`;
controls.appendChild(style);

let div = document.createElement("div");
div.setAttribute("id", "controls");
let button = document.createElement("button");
button.textContent = "Click me";
div.appendChild(button);
controls.appendChild(div);
console.log('INNER', div.innerHTML);
