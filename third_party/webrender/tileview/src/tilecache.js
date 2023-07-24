// current SVG file for scrubbing and playback
var svg_index = 0;

// double buffered <object>s each holding an SVG file
var backbuffer;
var frontbuffer;


// timer for animation
var svg_timer;
var is_playing = false;

function toggle_play() {
  if( is_playing ) {
    is_playing = false;
    clearInterval(svg_timer);
    document.getElementById("text_spacebar").innerHTML =
      'Spacebar to play';
  } else {
    is_playing = true;
    svg_timer = setInterval(on_tick, 100);
    document.getElementById("text_spacebar").innerHTML =
      'Playing (Spacebar to stop)';
    function on_tick() {
      if( svg_index + 1 == svg_files.length ) {
        toggle_play();
      } else {
        go_to_svg(svg_index+1);
      }
    }
  }
}

function toggle_quadtree() {
    var quad_groups = document.getElementsByClassName("svg_quadtree")
    var i;
    for (i = 0; i < quad_groups.length; i++) {
        if( quad_groups[i].style.display == "none" )
            quad_groups[i].style.display = "block";
        else
            quad_groups[i].style.display = "none";
    }
}

function update_slice_visibility(max_slice) {
	let content = frontbuffer.contentDocument;
	update_slice_visibility_for_content(content, max_slice);
}

function update_slice_visibility_for_content(content, max_slice) {

	intern = document.getElementById('intern').contentDocument;

	for (let slice = 0; slice != max_slice; ++slice) {
		var cbox_name = "slice_toggle" + slice;
		let cbox = document.getElementById(cbox_name);
		if( !cbox )
			continue;
		let checked = cbox.checked;
		if (content) { // might fail due to cross scripting -- use SimpleHTTPServer
			var id = "tile_slice" + slice + "_everything";
			var group = content.getElementById(id);
			if (group) {
				if (checked)
					group.style.display = "block";
				else
					group.style.display = "none";
			}
		}
		if (intern) {
			var id = "invalidation_slice" + slice;
			var div = intern.getElementById(id);
			if (div) {
				if (checked)
					div.style.display = "block";
				else
					div.style.display = "none";
			}
		}
	}
}

// try to block repeated keypressed from causing flickering
// when they land between go_to_svg returning and onload
// firing.
var is_loading = false;

function go_to_svg(index) {
  if( index >= svg_files.length ||
      index < 0 ||
      is_loading ) {
        return;
  }

  is_loading = true;
  svg_index = index;

  var slider = document.getElementById('frame_slider');
  // won't recurse thanks to is_loading
  slider.value = svg_index;

  backbuffer.onload = function(){

    document.getElementById("text_frame_counter").innerHTML =
      svg_files[svg_index];

	let content = backbuffer.contentDocument;
	update_slice_visibility_for_content(content, 20);

    backbuffer.style.display = '';
    frontbuffer.style.display = 'none';

    var t = frontbuffer;
    frontbuffer = backbuffer;
    backbuffer = t;
    is_loading = false;
  }
  document.getElementById('intern').src = intern_files[svg_index];
  backbuffer.setAttribute('data', svg_files[svg_index]);

  // also see https://stackoverflow.com/a/29915275
}

function load() {
  window.addEventListener('keypress', handle_keyboard_shortcut);
  window.addEventListener('keydown',  handle_keydown);

  frontbuffer = document.getElementById("svg_container0");
  backbuffer  = document.getElementById("svg_container1");
  backbuffer.style.display='none';

  var slider = document.getElementById('frame_slider');
  slider.oninput = function() {
    if( is_playing ) {
      toggle_play();
    }
    go_to_svg(this.value);
  }
}

function handle_keyboard_shortcut(event) {
  switch (event.charCode) {
    case 32: // ' '
      toggle_play();
      break;
    case 113: // 'q'
      toggle_quadtree();
      break;
		/*
  case 49: // "1" key
    document.getElementById("radio1").checked = true;
    show_image(1);
    break;
  case 50: // "2" key
    document.getElementById("radio2").checked = true;
    show_image(2);
    break;
  case 100: // "d" key
    document.getElementById("differences").click();
    break;
  case 112: // "p" key
    shift_images(-1);
    break;
  case 110: // "n" key
    shift_images(1);
    break;
  */
  }
}

function handle_keydown(event) {
  switch (event.keyCode) {
  case 37:  // left arrow
    go_to_svg(svg_index-1);
    event.preventDefault();
    break;
  case 38:  // up arrow
    break;
  case 39:  // right arrow
    go_to_svg(svg_index+1);
    event.preventDefault();
    break;
  case 40:  // down arrow
    break;
  }
}

