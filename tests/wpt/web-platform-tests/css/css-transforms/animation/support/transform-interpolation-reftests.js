'use strict';

// Each test is an array of [endpoint, midpoint, endpoint] and tests
// whether the endpoints interpolate to the same visual state as the midpoint
const transformTests = {
  translate: [
    ['translateX(0px)', 'translateX(25px)', 'translateX(50px)'],
    ['translateY(0px)', 'translateY(25px)', 'translateY(50px)'],
    ['translateX(0%)', 'translateX(25%)', 'translateX(50%)'],
    ['translateY(0%)', 'translateY(25%)', 'translateY(50%)'],
    ['translateX(50%)', 'translate(25%, 25%)', 'translateY(50%)'],
    ['translateX(50%)', 'translate(25%, 25px)', 'translateY(50px)'],
    ['translateX(50px)', 'translateX(calc(25px + 25%))', 'translateX(50%)']
  ],
  translateEm: [
    ['translateX(0em)', 'translateX(2em)', 'translateX(4em)'],
    ['translateX(-50px)', 'translateX(calc(2em - 25px))', 'translateX(4em)'],
    ['translateX(50%)', 'translateX(calc(25% - 2em))', 'translateX(-4em)']
  ],
  rotate: [
    ['rotate(0deg)', 'rotate(45deg)', 'rotate(90deg)'],
    ['rotateX(0deg)', 'rotateX(45deg)', 'rotateX(90deg)'],
    ['rotateY(0deg)', 'rotateY(45deg)', 'rotateY(90deg)'],
    ['rotate(0deg)', 'rotate(180deg)', 'rotate(360deg)'],
    ['rotate3d(7, 8, 9, 0deg)', 'rotate3d(7, 8, 9, 45deg)', 'rotate3d(7, 8, 9, 90deg)'],
    // First endpoint is the same rotation as rotateZ(0deg) but triggers SLERP
    ['rotateX(360deg)', 'rotateZ(45deg)', 'rotateZ(90deg)']
  ],
  scale: [
    ['scaleX(0.5)', 'scaleX(0.75)', 'scaleX(1)'],
    ['scaleY(0.5)', 'scaleY(0.75)', 'scaleY(1)'],
    ['scale(0.5)', 'scale(0.75)', 'scale(1)'],
    ['scaleX(0.5)', 'scale(0.75)', 'scaleY(0.5)'],
    ['scale3d(0.5, 1, 2)', 'scale3d(0.75, 0.75, 3)', 'scale3d(1, 0.5, 4)']
  ],
  skew: [
    ['skewX(0deg)', 'skewX(30deg)', 'skewX(60deg)'],
    ['skewY(0deg)', 'skewY(30deg)', 'skewY(60deg)'],
    ['skew(60deg, 0deg)', 'skew(30deg, 30deg)', 'skew(0deg, 60deg)'],
    ['skewX(0deg) rotate(0deg)', 'skewX(0deg) rotate(180deg)', 'skewX(0deg) rotate(360deg)'],
    ['skewX(0deg) rotate(0deg)', 'matrix(1, 0, 0, 1, 0, 0)', 'skewY(0deg) rotate(360deg)']
  ],
  matrix: [
    // matched matrix parameters do not collapse the values after them
    ['matrix(1,0,0,1,0,0) rotate(0deg)', 'matrix(1.5,0,0,1.5,0,0) rotate(180deg)', 'matrix(2,0,0,2,0,0) rotate(360deg)']
  ]
}

// Initial setup, which includes properties that will be overridden to
// test invalidation.
function initialStyle(div) {
  div.style.width = '180px';
  div.style.height = '150px';
  div.style.margin = '50px';
  div.style.borderLeft = 'solid 40px blue';
  div.style.backgroundColor = 'green';
  div.style.willChange = 'transform';
  div.style.fontSize = '30px';
}

function finalStyle(div) {
  div.style.width = '80px';
  div.style.height = '80px';
  div.style.fontSize = '15px';
}

function styleBody(){
  let body = document.body;
  body.style.display = 'flex';
  body.style.flexDirection = 'row';
  body.style.flexWrap = 'wrap';
}

// Simulate a static image at 50% progeress with a running animation.
// The easing curve has zero slope and curvature at its midpoint of 50% -> 50%.
// The timing values are chosen so as so that a delay of up to 10s will not
// cause a visual change.
const easing = 'cubic-bezier(0,1,1,0)';
const duration = 1e9;
const delay = -duration/2;

// Indices to unpack a test case, which is in the format
// [start, midpoint, end]
const startIndex = 0;
const midIndex = 1;
const endIndex = 2;

async function createTests(tests) {
  styleBody();
  for (const test of tests) {
    let div = document.createElement('div');
    document.body.appendChild(div);
    initialStyle(div);
    var anim = div.animate(
        {transform: [test[startIndex], test[endIndex]]},
        {duration: duration, delay: delay, easing: easing});
    await anim.ready;
    finalStyle(div);  // Change size to test invalidation.
  }

  await new Promise(requestAnimationFrame);
  await new Promise(requestAnimationFrame);
  takeScreenshot();
}

// Create references using an animation with identical keyframes for start
// and end so as to avoid rounding and anti-aliasing differences between
// animated and non-animated pathways.
async function createRefs(tests) {
  styleBody();
  for (const test of tests) {
    let div = document.createElement('div');
    document.body.appendChild(div);
    initialStyle(div);
    finalStyle(div);
    var anim = div.animate(
        {transform: [test[midIndex], test[midIndex]]},
        {duration: duration, delay: delay, easing: easing});
    await anim.ready;
  }

  await new Promise(requestAnimationFrame);
  await new Promise(requestAnimationFrame);
  takeScreenshot();
}

