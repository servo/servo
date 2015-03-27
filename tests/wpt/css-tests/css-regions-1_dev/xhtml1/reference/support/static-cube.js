// This source is the javascript needed to build a simple moving
// cube in **three.js** based on this
// [example](https://raw.github.com/mrdoob/three.js/r44/examples/canvas_geometry_cube.html)
// It is the source about this [blog post](/blog/2011/08/06/lets-do-a-cube/).

// ## Now lets start

// declare a bunch of variable we will need later
var container;
var camera, scene, renderer;
var cube;

// ## Initialize everything
function init() {
	// create the camera
	camera = new THREE.Camera( 70, 4/3, 100, 1000 );
	camera.position.z = 350;

	// create the Scene
	scene = new THREE.Scene();

	// create the Cube
	cube = new THREE.Mesh( new THREE.CubeGeometry( 200, 200, 200 ), new THREE.MeshNormalMaterial() );

	// add the object to the scene
	scene.addObject( cube );

	// create the container element
	container = document.querySelector("#container");

	// init the WebGL renderer and append it to the Dom
	renderer = new THREE.WebGLRenderer();
	renderer.setSize( container.getBoundingClientRect().width, container.getBoundingClientRect().height );
	container.appendChild( renderer.domElement );
}


// ## Render the 3D Scene
function render() {
	// animate the cube
	cube.rotation.x = 0.5;
	cube.rotation.y = 0.8;
	cube.rotation.z = 0.2;

	// actually display the scene in the DOM element
	renderer.render( scene, camera );
}

document.addEventListener("DOMContentLoaded", function() {
	init();
	render();
})