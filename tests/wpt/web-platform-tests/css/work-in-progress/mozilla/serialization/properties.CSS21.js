// True longhand properties.
const CSS_TYPE_LONGHAND = 0;

// True shorthand properties.
const CSS_TYPE_TRUE_SHORTHAND = 1;

// Properties that we handle as shorthands but were longhands either in
// the current spec or earlier versions of the spec.
const CSS_TYPE_SHORTHAND_AND_LONGHAND = 2;

// Each property has the following fields:
//	 domProp: The name of the relevant member of nsIDOM[NS]CSS2Properties
//	 inherited: Whether the property is inherited by default (stated as 
//	   yes or no in the property header in all CSS specs)
//	 type: see above
//	 get_computed: if present, the property's computed value shows up on
//	   another property, and this is a function used to get it
//	 initial_values: Values whose computed value should be the same as the
//	   computed value for the property's initial value.
//	 other_values: Values whose computed value should be different from the
//	   computed value for the property's initial value.
//	 XXX Should have a third field for values whose computed value may or
//	   may not be the same as for the property's initial value.
//	 invalid_values: Things that are not values for the property and
//	   should be rejected.
//	 FIXME: Add field for spec link.

// FIXME: use array.concat more rather than repeating myself.

// Helper functions used to construct gCSSProperties.

function initial_font_family_is_sans_serif()
{
	// The initial value of 'font-family' might be 'serif' or
	// 'sans-serif'.
	var div = document.createElement("div");
	div.setAttribute("style", "font: initial");
	return getComputedStyle(div, "").fontFamily == "sans-serif";
}
var gInitialFontFamilyIsSansSerif = initial_font_family_is_sans_serif();

var gCSSProperties = {
	"background": {
		domProp: "background",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "background-attachment", "background-color", "background-image", "background-position", "background-repeat", "background-clip", "background-origin", "background-size" ],
		initial_values: [ "transparent", "none", "repeat", "scroll", "0% 0%", "top left", "left top", "transparent none", "top left none", "left top none", "none left top", "none top left", "none 0% 0%", "transparent none repeat scroll top left", "left top repeat none scroll transparent" ],
		other_values: [
				/* without multiple backgrounds */
			"green",
			"none green repeat scroll left top",
			"url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==)",
			"repeat url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==') transparent left top scroll",
			"repeat-x",
			"repeat-y",
			"no-repeat",
			"none repeat-y transparent scroll 0% 0%",
			"fixed",
			"0% top transparent fixed repeat none",
			"top",
			"left",
			"50% 50%",
			"center",
			"bottom right scroll none transparent repeat",
			"50% transparent",
			"transparent 50%",
			"50%",
		],
		invalid_values: [
			/* mixes with keywords have to be in correct order */
			"50% left", "top 50%",
			/* bug 258080: don't accept background-position separated */
			"left url(404.png) top", "top url(404.png) left",
		]
	},
	"background-attachment": {
		domProp: "backgroundAttachment",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "scroll" ],
		other_values: [ "fixed" ],
		invalid_values: []
	},
	"background-color": {
		domProp: "backgroundColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "transparent" ],
		other_values: [ "green", "rgb(255, 0, 128)", "#fc2", "#96ed2a", "black" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000", "rgb(255.0,0.387,3489)" ]
	},
	"background-image": {
		domProp: "backgroundImage",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [
		"url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==)", "url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==')", 'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
		],
		invalid_values: [ ]
	},
	"background-position": {
		domProp: "backgroundPosition",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* is "0px 0px" an initial value or not? */
		initial_values: [ "top left", "left top", "0% 0%", "0% top", "left 0%", "0px 0px" ],
		other_values: [ "top", "left", "right", "bottom", "center", "center bottom", "bottom center", "center right", "right center", "center top", "top center", "center left", "left center", "right bottom", "bottom right", "50%", "0%", "0px", "30px" ],
		invalid_values: [ "50% left", "top 50%" ]
	},
	"background-repeat": {
		domProp: "backgroundRepeat",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "repeat" ],
		other_values: [ "repeat-x", "repeat-y", "no-repeat" ],
		invalid_values: [ ]
	},
	"border": {
		domProp: "border",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-bottom-color", "border-bottom-style", "border-bottom-width", "border-left-color", "border-left-style", "border-left-width", "border-right-color", "border-right-style", "border-right-width", "border-top-color", "border-top-style", "border-top-width" ],
		initial_values: [ "none", "medium", "thin" ],
		other_values: [ "solid", "medium solid", "green solid", "10px solid", "thick solid" ],
		invalid_values: [ "5%" ]
	},
	"border-bottom": {
		domProp: "borderBottom",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-bottom-color", "border-bottom-style", "border-bottom-width" ],
		initial_values: [ "none", "medium", "thin" ],
		other_values: [ "solid", "green", "medium solid", "green solid", "10px solid", "thick solid", "5px green none" ],
		invalid_values: [ "5%" ]
	},
	"border-bottom-color": {
		domProp: "borderBottomColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "black" },
		initial_values: [ "black" ],
		other_values: [ "green", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"border-bottom-style": {
		domProp: "borderBottomStyle",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX hidden is sometimes the same as initial */
		initial_values: [ "none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge" ],
		invalid_values: []
	},
	"border-bottom-width": {
		domProp: "borderBottomWidth",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "border-bottom-style": "solid" },
		initial_values: [ "medium", "3px" ],
		other_values: [ "thin", "thick", "1px", "2em" ],
		invalid_values: [ "5%" ]
	},
	"border-collapse": {
		domProp: "borderCollapse",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "separate" ],
		other_values: [ "collapse" ],
		invalid_values: []
	},
	"border-color": {
		domProp: "borderColor",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-top-color", "border-right-color", "border-bottom-color", "border-left-color" ],
		prerequisites: { "color": "black" },
		initial_values: [ "black" ],
		other_values: [ "green", "black green", "black black green", "black black black green", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"border-left": {
		domProp: "borderLeft",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-left-color", "border-left-style", "border-left-width" ],
		initial_values: [ "none", "medium", "black", "thin", "none medium black" ],
		other_values: [ "solid", "green", "medium solid", "green solid", "10px solid", "thick solid", "5px green none" ],
		invalid_values: [ "5%" ]
	},
	"border-left-color": {
		domProp: "borderLeftColor",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		prerequisites: { "color": "black" },
		initial_values: [ "black" ],
		other_values: [ "green", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"border-left-style": {
		domProp: "borderLeftStyle",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* XXX hidden is sometimes the same as initial */
		initial_values: [ "none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge" ],
		invalid_values: []
	},
	"border-left-width": {
		domProp: "borderLeftWidth",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		prerequisites: { "border-left-style": "solid" },
		initial_values: [ "medium", "3px" ],
		other_values: [ "thin", "thick", "1px", "2em" ],
		invalid_values: [ "5%" ]
	},
	"border-right": {
		domProp: "borderRight",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-right-color", "border-right-style", "border-right-width" ],
		prerequisites: { "color": "black" },
		initial_values: [ "none", "medium", "black", "thin", "none medium black" ],
		other_values: [ "solid", "green", "medium solid", "green solid", "10px solid", "thick solid", "5px green none" ],
		invalid_values: [ "5%" ]
	},
	"border-right-color": {
		domProp: "borderRightColor",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		prerequisites: { "color": "black" },
		initial_values: [ "black" ],
		other_values: [ "green", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"border-right-style": {
		domProp: "borderRightStyle",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* XXX hidden is sometimes the same as initial */
		initial_values: [ "none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge" ],
		invalid_values: []
	},
	"border-right-width": {
		domProp: "borderRightWidth",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		prerequisites: { "border-right-style": "solid" },
		initial_values: [ "medium", "3px" ],
		other_values: [ "thin", "thick", "1px", "2em" ],
		invalid_values: [ "5%" ]
	},
	"border-spacing": {
		domProp: "borderSpacing",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "0 0", "0px", "0 0px" ],
		other_values: [ "3px", "4em 2px", "4em 0", "0px 2px" ],
		invalid_values: [ "0%", "0 0%", "-5px", "-5px -5px", "0 -5px", "-5px 0" ]
	},
	"border-style": {
		domProp: "borderStyle",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-top-style", "border-right-style", "border-bottom-style", "border-left-style" ],
		/* XXX hidden is sometimes the same as initial */
		initial_values: [ "none", "none none", "none none none", "none none none none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge", "none solid", "none none solid", "none none none solid", "groove none none none", "none ridge none none", "none none double none", "none none none dotted" ],
		invalid_values: []
	},
	"border-top": {
		domProp: "borderTop",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-top-color", "border-top-style", "border-top-width" ],
		prerequisites: { "color": "black" },
		initial_values: [ "none", "medium", "black", "thin", "none medium black" ],
		other_values: [ "solid", "green", "medium solid", "green solid", "10px solid", "thick solid", "5px green none" ],
		invalid_values: [ "5%" ]
	},
	"border-top-color": {
		domProp: "borderTopColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "black" },
		initial_values: [ "black" ],
		other_values: [ "green", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"border-top-style": {
		domProp: "borderTopStyle",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX hidden is sometimes the same as initial */
		initial_values: [ "none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge" ],
		invalid_values: []
	},
	"border-top-width": {
		domProp: "borderTopWidth",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "border-top-style": "solid" },
		initial_values: [ "medium", "3px" ],
		other_values: [ "thin", "thick", "1px", "2em" ],
		invalid_values: [ "5%" ]
	},
	"border-width": {
		domProp: "borderWidth",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "border-top-width", "border-right-width", "border-bottom-width", "border-left-width" ],
		prerequisites: { "border-style": "solid" },
		initial_values: [ "medium", "3px", "medium medium", "3px medium medium", "medium 3px medium medium" ],
		other_values: [ "thin", "thick", "1px", "2em", "2px 0 0px 1em" ],
		invalid_values: [ "5%" ]
	},
	"bottom": {
		domProp: "bottom",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* FIXME: run tests with multiple prerequisites */
		prerequisites: { "position": "relative" },
		/* XXX 0 may or may not be equal to auto */
		initial_values: [ "auto" ],
		other_values: [ "32px", "-3em", "12%" ],
		invalid_values: []
	},
	"caption-side": {
		domProp: "captionSide",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "top" ],
		other_values: [ "bottom" ],
		invalid_values: []
	},
	"clear": {
		domProp: "clear",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "left", "right", "both" ],
		invalid_values: []
	},
	"clip": {
		domProp: "clip",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "rect(0 0 0 0)", "rect(auto,auto,auto,auto)", "rect(3px, 4px, 4em, 0)", "rect(auto, 3em, 4pt, 2px)", "rect(2px 3px 4px 5px)" ],
		invalid_values: [ "rect(auto, 3em, 2%, 5px)", "rect(0 0, 0 0)" ]
	},
	"color": {
		domProp: "color",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "black", "#000" ],
		other_values: [ "green", "#f3c", "#fed292", "transparent" ],
		invalid_values: [ "fff", "ffffff", "#f", "#ff", "#ffff", "#fffff", "#fffffff", "#ffffffff", "#fffffffff" ]
	},
	"content": {
		domProp: "content",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX needs to be on pseudo-elements */
		initial_values: [ "normal", "none" ],
		other_values: [ '""', "''", '"hello"', "url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==)", "url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==')", 'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")', 'counter(foo)', 'counter(bar, upper-roman)', 'counters(foo, ".")', "counters(bar, '-', lower-greek)", "'-' counter(foo) '.'", "attr(title)", "open-quote", "close-quote", "no-open-quote", "no-close-quote", "close-quote attr(title) counters(foo, '.', upper-alpha)", "counter(foo, none)", "counters(bar, '.', none)", "attr(\\32)", "attr(\\2)", "attr(-\\2)", "attr(-\\32)", "counter(\\2)", "counters(\\32, '.')", "counter(-\\32, upper-roman)", "counters(-\\2, '-', lower-greek)", "counter(\\()", "counters(a\\+b, '.')", "counter(\\}, upper-alpha)", "-moz-alt-content" ],
		invalid_values: [ 'counters(foo)', 'counter(foo, ".")', 'attr("title")', "attr('title')", "attr(2)", "attr(-2)", "counter(2)", "counters(-2, '.')", "-moz-alt-content 'foo'", "'foo' -moz-alt-content" ]
	},
	"counter-increment": {
		domProp: "counterIncrement",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "foo 1", "bar", "foo 3 bar baz 2", "\\32  1", "-\\32  1", "-c 1", "\\32 1", "-\\32 1", "\\2  1", "-\\2  1", "-c 1", "\\2 1", "-\\2 1" ],
		invalid_values: []
	},
	"counter-reset": {
		domProp: "counterReset",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "foo 1", "bar", "foo 3 bar baz 2", "\\32  1", "-\\32  1", "-c 1", "\\32 1", "-\\32 1", "\\2  1", "-\\2  1", "-c 1", "\\2 1", "-\\2 1" ],
		invalid_values: []
	},
	"cursor": {
		domProp: "cursor",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "crosshair", "default", "pointer", "move", "e-resize", "ne-resize", "nw-resize", "n-resize", "se-resize", "sw-resize", "s-resize", "w-resize", "text", "wait", "help", "progress" ],
		invalid_values: []
	},
	"direction": {
		domProp: "direction",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "ltr" ],
		other_values: [ "rtl" ],
		invalid_values: []
	},
	"display": {
		domProp: "display",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "inline" ],
		/* XXX none will really mess with other properties */
		prerequisites: { "float": "none", "position": "static" },
		other_values: [ "block", "list-item", "inline-block", "table", "inline-table", "table-row-group", "table-header-group", "table-footer-group", "table-row", "table-column-group", "table-column", "table-cell", "table-caption", "none" ],
		invalid_values: []
	},
	"empty-cells": {
		domProp: "emptyCells",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "show" ],
		other_values: [ "hide" ],
		invalid_values: []
	},
	"float": {
		domProp: "cssFloat",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "left", "right" ],
		invalid_values: []
	},
	"font": {
		domProp: "font",
		inherited: true,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "font-style", "font-variant", "font-weight", "font-size", "line-height", "font-family" ],
		initial_values: [ (gInitialFontFamilyIsSansSerif ? "medium sans-serif" : "medium serif") ],
		other_values: [ "large serif", "9px fantasy", "bold italic small-caps 24px/1.4 Times New Roman, serif", "caption", "icon", "menu", "message-box", "small-caption", "status-bar" ],
		invalid_values: []
	},
	"font-family": {
		domProp: "fontFamily",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ (gInitialFontFamilyIsSansSerif ? "sans-serif" : "serif") ],
		other_values: [ (gInitialFontFamilyIsSansSerif ? "serif" : "sans-serif"), "Times New Roman, serif", "'Times New Roman', serif", "cursive", "fantasy", "\"Times New Roman", "Times, \"Times New Roman" ],
		invalid_values: [ "\"Times New\" Roman" ]
	},
	"font-size": {
		domProp: "fontSize",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "medium" ],
		other_values: [ "large", "2em", "50%", "xx-small", "36pt", "8px", "0px", "0%" ],
		invalid_values: [ "-2em", "-50%", "-1px" ]
	},
	"font-style": {
		domProp: "fontStyle",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "italic", "oblique" ],
		invalid_values: []
	},
	"font-variant": {
		domProp: "fontVariant",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "small-caps" ],
		invalid_values: []
	},
	"font-weight": {
		domProp: "fontWeight",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal", "400" ],
		other_values: [ "bold", "100", "200", "300", "500", "600", "700", "800", "900", "bolder", "lighter" ],
		invalid_values: [ "107", "399", "401", "699", "710" ]
	},
	"height": {
		domProp: "height",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* FIXME: test zero, and test calc clamping */
		initial_values: [ " auto" ],
		/* computed value tests for height test more with display:block */
		prerequisites: { "display": "block" },
		other_values: [ "15px", "3em", "15%" ],
		invalid_values: [ "none" ]
	},
	"left": {
		domProp: "left",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* FIXME: run tests with multiple prerequisites */
		prerequisites: { "position": "relative" },
		/* XXX 0 may or may not be equal to auto */
		initial_values: [ "auto" ],
		other_values: [ "32px", "-3em", "12%" ],
		invalid_values: []
	},
	"letter-spacing": {
		domProp: "letterSpacing",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "0", "0px", "1em", "2px", "-3px" ],
		invalid_values: []
	},
	"line-height": {
		domProp: "lineHeight",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		/*
		 * Inheritance tests require consistent font size, since
		 * getComputedStyle (which uses the CSS2 computed value, or
		 * CSS2.1 used value) doesn't match what the CSS2.1 computed
		 * value is.  And they even require consistent font metrics for
		 * computation of 'normal'.
		 */
		prerequisites: { "font-size": "19px", "font-size-adjust": "none", "font-family": "serif", "font-weight": "normal", "font-style": "normal" },
		initial_values: [ "normal" ],
		other_values: [ "1.0", "1", "1em", "47px" ],
		invalid_values: []
	},
	"list-style": {
		domProp: "listStyle",
		inherited: true,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "list-style-type", "list-style-position", "list-style-image" ],
		initial_values: [ "outside", "disc", "disc outside", "outside disc", "disc none", "none disc", "none disc outside", "none outside disc", "disc none outside", "disc outside none", "outside none disc", "outside disc none" ],
		other_values: [ "inside none", "none inside", "none none inside", "square", "none", "none none", "outside none none", "none outside none", "none none outside", "none outside", "outside none",
			'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			'none url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") none',
			'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") outside',
			'outside url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			'outside none url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			'outside url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") none',
			'none url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") outside',
			'none outside url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") outside none',
			'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==") none outside'
		],
		invalid_values: [ "outside outside", "disc disc", "unknown value", "none none none", "none disc url(404.png)", "none url(404.png) disc", "disc none url(404.png)", "disc url(404.png) none", "url(404.png) none disc", "url(404.png) disc none", "none disc outside url(404.png)" ]
	},
	"list-style-image": {
		domProp: "listStyleImage",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ 'url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAKElEQVR42u3NQQ0AAAgEoNP+nTWFDzcoQE1udQQCgUAgEAgEAsGTYAGjxAE/G/Q2tQAAAABJRU5ErkJggg==")',
			// Add some tests for interesting url() values here to test serialization, etc.
			"url(\'data:text/plain,\"\')",
			"url(\"data:text/plain,\'\")",
			"url(\'data:text/plain,\\\'\')",
			"url(\"data:text/plain,\\\"\")",
			"url(\'data:text/plain,\\\"\')",
			"url(\"data:text/plain,\\\'\")",
			"url(data:text/plain,\\\\)",
		],
		invalid_values: []
	},
	"list-style-position": {
		domProp: "listStylePosition",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "outside" ],
		other_values: [ "inside" ],
		invalid_values: []
	},
	"list-style-type": {
		domProp: "listStyleType",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "disc" ],
		other_values: [ "circle", "decimal-leading-zero", "upper-alpha" ],
		invalid_values: []
	},
	"margin": {
		domProp: "margin",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "margin-top", "margin-right", "margin-bottom", "margin-left" ],
		initial_values: [ "0", "0px 0 0em", "0% 0px 0em 0pt" ],
		other_values: [ "3px 0", "2em 4px 2pt", "1em 2em 3px 4px" ],
		invalid_values: []
	},
	"margin-bottom": {
		domProp: "marginBottom",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX testing auto has prerequisites */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"margin-left": {
		domProp: "marginLeft",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* no subproperties */
		/* XXX testing auto has prerequisites */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)" ],
		other_values: [ "1px", "2em", "5%", ".5px", "+32px", "+.789px", "-.328px", "+0.56px", "-0.974px", "237px", "-289px", "-056px", "1987.45px", "-84.32px",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ "..25px", ".+5px", ".px", "-.px", "++5px", "-+4px", "+-3px", "--7px", "+-.6px", "-+.5px", "++.7px", "--.4px" ]
	},
	"margin-right": {
		domProp: "marginRight",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* no subproperties */
		/* XXX testing auto has prerequisites */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"margin-top": {
		domProp: "marginTop",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX testing auto has prerequisites */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"marker-offset": {
		domProp: "markerOffset",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "6em", "-1px", "-moz-calc(0px)", "-moz-calc(3em + 2px - 4px)", "-moz-calc(-2em)" ],
		invalid_values: []
	},
	"max-height": {
		domProp: "maxHeight",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block" },
		initial_values: [ "none" ],
		other_values: [ "30px", "50%", "0",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(0px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ "auto", "-moz-max-content", "-moz-min-content", "-moz-fit-content", "-moz-available" ]
	},
	"max-width": {
		domProp: "maxWidth",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block" },
		initial_values: [ "none" ],
		other_values: [ "30px", "50%", "0", "-moz-max-content", "-moz-min-content", "-moz-fit-content", "-moz-available",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(0px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ "auto" ]
	},
	"min-height": {
		domProp: "minHeight",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block" },
		initial_values: [ "0", "-moz-calc(0em)", "-moz-calc(-2px)", "-moz-calc(-1%)" ],
		other_values: [ "30px", "50%",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ "auto", "none", "-moz-max-content", "-moz-min-content", "-moz-fit-content", "-moz-available" ]
	},
	"min-width": {
		domProp: "minWidth",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block" },
		initial_values: [ "0", "-moz-calc(0em)", "-moz-calc(-2px)", "-moz-calc(-1%)" ],
		other_values: [ "30px", "50%", "-moz-max-content", "-moz-min-content", "-moz-fit-content", "-moz-available",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ "auto", "none" ]
	},
	"opacity": {
		domProp: "opacity",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1", "17", "397.376" ],
		other_values: [ "0", "0.4", "0.0000", "-3" ],
		invalid_values: [ "0px", "1px" ]
	},
	"orphans": {
		domProp: "orphans",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		// XXX requires display:block
		initial_values: [ "2" ],
		other_values: [ "1", "7" ],
		invalid_values: [ "0", "-1", "0px", "3px" ]
	},
	"outline": {
		domProp: "outline",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "outline-color", "outline-style", "outline-width" ],
		initial_values: [
			"none", "medium", "thin",
			// XXX Should be invert, but currently currentcolor.
			//"invert", "none medium invert"
			"currentColor", "none medium currentcolor"
		],
		other_values: [ "solid", "medium solid", "green solid", "10px solid", "thick solid" ],
		invalid_values: [ "5%" ]
	},
	"outline-color": {
		domProp: "outlineColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "black" },
		initial_values: [ "currentColor" ], // XXX should be invert
		other_values: [ "green", "rgba(255,128,0,0.5)", "transparent" ],
		invalid_values: [ "#0", "#00", "#0000", "#00000", "#0000000", "#00000000", "#000000000" ]
	},
	"outline-offset": {
		domProp: "outlineOffset",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "0px", "-0", "-moz-calc(0px)", "-moz-calc(3em + 2px - 2px - 3em)", "-moz-calc(-0em)" ],
		other_values: [ "-3px", "1em", "-moz-calc(3em)", "-moz-calc(7pt + 3 * 2em)", "-moz-calc(-3px)" ],
		invalid_values: [ "5%" ]
	},
	"outline-style": {
		domProp: "outlineStyle",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		// XXX Should 'hidden' be the same as initial?
		initial_values: [ "none" ],
		other_values: [ "solid", "dashed", "dotted", "double", "outset", "inset", "groove", "ridge" ],
		invalid_values: []
	},
	"outline-width": {
		domProp: "outlineWidth",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "outline-style": "solid" },
		initial_values: [ "medium", "3px", "-moz-calc(4px - 1px)" ],
		other_values: [ "thin", "thick", "1px", "2em",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(0px)",
			"-moz-calc(0px)",
			"-moz-calc(5em)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 5em)",
		],
		invalid_values: [ "5%" ]
	},
	"overflow": {
		domProp: "overflow",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		prerequisites: { "display": "block" },
		subproperties: [ "overflow-x", "overflow-y" ],
		initial_values: [ "visible" ],
		other_values: [ "auto", "scroll", "hidden" ],
		invalid_values: []
	},
	"overflow-x": {
		domProp: "overflowX",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block", "overflow-y": "visible" },
		initial_values: [ "visible" ],
		other_values: [ "auto", "scroll", "hidden" ],
		invalid_values: []
	},
	"overflow-y": {
		domProp: "overflowY",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "display": "block", "overflow-x": "visible" },
		initial_values: [ "visible" ],
		other_values: [ "auto", "scroll", "hidden" ],
		invalid_values: []
	},
	"padding": {
		domProp: "padding",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "padding-top", "padding-right", "padding-bottom", "padding-left" ],
		initial_values: [ "0", "0px 0 0em", "0% 0px 0em 0pt", "-moz-calc(0px) -moz-calc(0em) -moz-calc(-2px) -moz-calc(-1%)" ],
		other_values: [ "3px 0", "2em 4px 2pt", "1em 2em 3px 4px" ],
		invalid_values: []
	},
	"padding-bottom": {
		domProp: "paddingBottom",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)", "-moz-calc(-3px)", "-moz-calc(-1%)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"padding-left": {
		domProp: "paddingLeft",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* no subproperties */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)", "-moz-calc(-3px)", "-moz-calc(-1%)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"padding-right": {
		domProp: "paddingRight",
		inherited: false,
		type: CSS_TYPE_SHORTHAND_AND_LONGHAND,
		/* no subproperties */
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)", "-moz-calc(-3px)", "-moz-calc(-1%)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"padding-top": {
		domProp: "paddingTop",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "0px", "0%", "-moz-calc(0pt)", "-moz-calc(0% + 0px)", "-moz-calc(-3px)", "-moz-calc(-1%)" ],
		other_values: [ "1px", "2em", "5%",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: [ ]
	},
	"page": {
		domProp: "page",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "foo", "bar" ],
		invalid_values: [ "3px" ]
	},
	"page-break-after": {
		domProp: "pageBreakAfter",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "always", "avoid", "left", "right" ],
		invalid_values: []
	},
	"page-break-before": {
		domProp: "pageBreakBefore",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "always", "avoid", "left", "right" ],
		invalid_values: []
	},
	"page-break-inside": {
		domProp: "pageBreakInside",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "avoid" ],
		invalid_values: []
	},
	"pointer-events": {
		domProp: "pointerEvents",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "visiblePainted", "visibleFill", "visibleStroke", "visible",
						"painted", "fill", "stroke", "all", "none" ],
		invalid_values: []
	},
	"position": {
		domProp: "position",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "static" ],
		other_values: [ "relative", "absolute", "fixed" ],
		invalid_values: []
	},
	"quotes": {
		domProp: "quotes",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ '"\u201C" "\u201D" "\u2018" "\u2019"',
						  '"\\201C" "\\201D" "\\2018" "\\2019"' ],
		other_values: [ "none", "'\"' '\"'" ],
		invalid_values: []
	},
	"right": {
		domProp: "right",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* FIXME: run tests with multiple prerequisites */
		prerequisites: { "position": "relative" },
		/* XXX 0 may or may not be equal to auto */
		initial_values: [ "auto" ],
		other_values: [ "32px", "-3em", "12%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: []
	},
	"table-layout": {
		domProp: "tableLayout",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "fixed" ],
		invalid_values: []
	},
	"text-align": {
		domProp: "textAlign",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		// don't know whether left and right are same as start
		initial_values: [ "start" ],
		other_values: [ "center", "justify", "end" ],
		invalid_values: []
	},
	"text-decoration": {
		domProp: "textDecoration",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "underline", "overline", "line-through", "blink line-through underline", "underline overline line-through blink", "-moz-anchor-decoration", "blink -moz-anchor-decoration" ],
		invalid_values: [ "underline none", "none underline", "line-through blink line-through" ]
	},
	"text-indent": {
		domProp: "textIndent",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "-moz-calc(3em - 5em + 2px + 2em - 2px)" ],
		other_values: [ "2em", "5%", "-10px",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: []
	},
	"text-shadow": {
		domProp: "textShadow",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "blue" },
		initial_values: [ "none" ],
		other_values: [ "2px 2px", "2px 2px 1px", "2px 2px green", "2px 2px 1px green", "green 2px 2px", "green 2px 2px 1px", "green 2px 2px, blue 1px 3px 4px", "currentColor 3px 3px", "blue 2px 2px, currentColor 1px 2px",
			/* calc() values */
			"2px 2px -moz-calc(-5px)", /* clamped */
			"-moz-calc(3em - 2px) 2px green",
			"green -moz-calc(3em - 2px) 2px",
			"2px -moz-calc(2px + 0.2em)",
			"blue 2px -moz-calc(2px + 0.2em)",
			"2px -moz-calc(2px + 0.2em) blue",
			"-moz-calc(-2px) -moz-calc(-2px)",
			"-2px -2px",
			"-moz-calc(2px) -moz-calc(2px)",
			"-moz-calc(2px) -moz-calc(2px) -moz-calc(2px)",
		],
		invalid_values: [ "3% 3%", "2px 2px -5px", "2px 2px 2px 2px", "2px 2px, none", "none, 2px 2px", "inherit, 2px 2px", "2px 2px, inherit",
			"-moz-calc(2px) -moz-calc(2px) -moz-calc(2px) -moz-calc(2px)"
		]
	},
	"text-transform": {
		domProp: "textTransform",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "capitalize", "uppercase", "lowercase" ],
		invalid_values: []
	},
	"top": {
		domProp: "top",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* FIXME: run tests with multiple prerequisites */
		prerequisites: { "position": "relative" },
		/* XXX 0 may or may not be equal to auto */
		initial_values: [ "auto" ],
		other_values: [ "32px", "-3em", "12%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: []
	},
	"-moz-transition": {
		domProp: "MozTransition",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "-moz-transition-property", "-moz-transition-duration", "-moz-transition-timing-function", "-moz-transition-delay" ],
		initial_values: [ "all 0s ease 0s" ],
		other_values: [ "width 1s linear 2s", "width 1s 2s linear", "width linear 1s 2s", "linear width 1s 2s", "linear 1s width 2s", "linear 1s 2s width", "1s width linear 2s", "1s width 2s linear", "1s 2s width linear", "1s linear width 2s", "1s linear 2s width", "1s 2s linear width", "width linear 1s", "width 1s linear", "linear width 1s", "linear 1s width", "1s width linear", "1s linear width", "1s 2s width", "1s width 2s", "width 1s 2s", "1s 2s linear", "1s linear 2s", "linear 1s 2s", "width 1s", "1s width", "linear 1s", "1s linear", "1s 2s", "2s 1s", "width", "linear", "1s", "height", "2s", "ease-in-out", "2s ease-in", "opacity linear", "ease-out 2s", "2s color, 1s width, 500ms height linear, 1s opacity 4s cubic-bezier(0.0, 0.1, 1.0, 1.0)", "1s \\32width linear 2s", "1s -width linear 2s", "1s -\\32width linear 2s", "1s \\32 0width linear 2s", "1s -\\32 0width linear 2s", "1s \\2width linear 2s", "1s -\\2width linear 2s" ],
		invalid_values: [ "2s, 1s width", "1s width, 2s", "2s all, 1s width", "1s width, 2s all", "1s width, 2s none", "2s none, 1s width", "2s inherit", "inherit 2s", "2s width, 1s inherit", "2s inherit, 1s width", "2s initial", "2s all, 1s width", "2s width, 1s all" ]
	},
	"-moz-transition-delay": {
		domProp: "MozTransitionDelay",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0s", "0ms" ],
		other_values: [ "1s", "250ms", "-100ms", "-1s", "1s, 250ms, 2.3s"],
		invalid_values: [ "0", "0px" ]
	},
	"-moz-transition-duration": {
		domProp: "MozTransitionDuration",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0s", "0ms" ],
		other_values: [ "1s", "250ms", "-1ms", "-2s", "1s, 250ms, 2.3s"],
		invalid_values: [ "0", "0px" ]
	},
	"-moz-transition-property": {
		domProp: "MozTransitionProperty",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "all" ],
		other_values: [ "none", "left", "top", "color", "width, height, opacity", "foobar", "auto", "\\32width", "-width", "-\\32width", "\\32 0width", "-\\32 0width", "\\2width", "-\\2width" ],
		invalid_values: [ "none, none", "all, all", "color, none", "none, color", "all, color", "color, all", "inherit, color", "color, inherit", "initial, color", "color, initial", "none, color", "color, none", "all, color", "color, all" ]
	},
	"-moz-transition-timing-function": {
		domProp: "MozTransitionTimingFunction",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "ease", "cubic-bezier(0.25, 0.1, 0.25, 1.0)" ],
		other_values: [ "linear", "ease-in", "ease-out", "ease-in-out", "linear, ease-in, cubic-bezier(0.1, 0.2, 0.8, 0.9)", "cubic-bezier(0.5, 0.5, 0.5, 0.5)", "cubic-bezier(0.25, 1.5, 0.75, -0.5)" ],
		invalid_values: [ "none", "auto", "cubic-bezier(0.25, 0.1, 0.25)", "cubic-bezier(0.25, 0.1, 0.25, 0.25, 1.0)", "cubic-bezier(-0.5, 0.5, 0.5, 0.5)", "cubic-bezier(1.5, 0.5, 0.5, 0.5)", "cubic-bezier(0.5, 0.5, -0.5, 0.5)", "cubic-bezier(0.5, 0.5, 1.5, 0.5)" ]
	},
	"unicode-bidi": {
		domProp: "unicodeBidi",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "embed", "bidi-override" ],
		invalid_values: [ "auto", "none" ]
	},
	"vertical-align": {
		domProp: "verticalAlign",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "baseline" ],
		other_values: [ "sub", "super", "top", "text-top", "middle", "bottom", "text-bottom", "15%", "3px", "0.2em", "-5px", "-3%",
			"-moz-calc(2px)",
			"-moz-calc(-2px)",
			"-moz-calc(50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(3*25px + 50%)",
		],
		invalid_values: []
	},
	"visibility": {
		domProp: "visibility",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "visible" ],
		other_values: [ "hidden", "collapse" ],
		invalid_values: []
	},
	"white-space": {
		domProp: "whiteSpace",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "pre", "nowrap", "pre-wrap", "pre-line" ],
		invalid_values: []
	},
	"widows": {
		domProp: "widows",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		// XXX requires display:block
		initial_values: [ "2" ],
		other_values: [ "1", "7" ],
		invalid_values: [ "0", "-1", "0px", "3px" ]
	},
	"width": {
		domProp: "width",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* computed value tests for width test more with display:block */
		prerequisites: { "display": "block" },
		initial_values: [ " auto" ],
		/* XXX these have prerequisites */
		other_values: [ "15px", "3em", "15%", "-moz-max-content", "-moz-min-content", "-moz-fit-content", "-moz-available",
			/* valid calc() values */
			"-moz-calc(-2px)",
			"-moz-calc(2px)",
			"-moz-calc(50%)",
			"-moz-calc(50% + 2px)",
			"-moz-calc( 50% + 2px)",
			"-moz-calc(50% + 2px )",
			"-moz-calc( 50% + 2px )",
			"-moz-calc(50% - -2px)",
			"-moz-calc(2px - -50%)",
			"-moz-calc(3*25px)",
			"-moz-calc(3 *25px)",
			"-moz-calc(3 * 25px)",
			"-moz-calc(3* 25px)",
			"-moz-calc(25px*3)",
			"-moz-calc(25px *3)",
			"-moz-calc(25px* 3)",
			"-moz-calc(25px * 3)",
			"-moz-calc(3*25px + 50%)",
			"-moz-calc(50% - 3em + 2px)",
			"-moz-calc(50% - (3em + 2px))",
			"-moz-calc((50% - 3em) + 2px)",
			"-moz-calc(2em)",
			"-moz-calc(50%)",
			"-moz-calc(50px/2)",
			"-moz-calc(50px/(2 - 1))"
		],
		invalid_values: [ "none", "-2px",
			/* invalid calc() values */
			"-moz-calc(50%+ 2px)",
			"-moz-calc(50% +2px)",
			"-moz-calc(50%+2px)",
			"-moz-min()",
			"-moz-calc(min())",
			"-moz-max()",
			"-moz-calc(max())",
			"-moz-min(5px)",
			"-moz-calc(min(5px))",
			"-moz-max(5px)",
			"-moz-calc(max(5px))",
			"-moz-min(5px,2em)",
			"-moz-calc(min(5px,2em))",
			"-moz-max(5px,2em)",
			"-moz-calc(max(5px,2em))",
			"-moz-calc(50px/(2 - 2))",
			/* If we ever support division by values, which is
			 * complicated for the reasons described in
			 * http://lists.w3.org/Archives/Public/www-style/2010Jan/0007.html
			 * , we should support all 4 of these as described in
			 * http://lists.w3.org/Archives/Public/www-style/2009Dec/0296.html
			 */
			"-moz-calc((3em / 100%) * 3em)",
			"-moz-calc(3em / 100% * 3em)",
			"-moz-calc(3em * (3em / 100%))",
			"-moz-calc(3em * 3em / 100%)"
		]
	},
	"word-spacing": {
		domProp: "wordSpacing",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal", "0", "0px", "-0em",
			"-moz-calc(-0px)", "-moz-calc(0em)"
		],
		other_values: [ "1em", "2px", "-3px",
			"-moz-calc(1em)", "-moz-calc(1em + 3px)",
			"-moz-calc(15px / 2)", "-moz-calc(15px/2)",
			"-moz-calc(-2em)"
		],
		invalid_values: []
	},
	"word-wrap": {
		domProp: "wordWrap",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "break-word" ],
		invalid_values: []
	},
	"z-index": {
		domProp: "zIndex",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		/* XXX requires position */
		initial_values: [ "auto" ],
		other_values: [ "0", "3", "-7000", "12000" ],
		invalid_values: [ "3.0", "17.5" ]
	}
	,
	"clip-path": {
		domProp: "clipPath",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#mypath)", "url('404.svg#mypath')" ],
		invalid_values: []
	},
	"clip-rule": {
		domProp: "clipRule",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "nonzero" ],
		other_values: [ "evenodd" ],
		invalid_values: []
	},
	"color-interpolation": {
		domProp: "colorInterpolation",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "sRGB" ],
		other_values: [ "auto", "linearRGB" ],
		invalid_values: []
	},
	"color-interpolation-filters": {
		domProp: "colorInterpolationFilters",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "linearRGB" ],
		other_values: [ "sRGB", "auto" ],
		invalid_values: []
	},
	"dominant-baseline": {
		domProp: "dominantBaseline",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "use-script", "no-change", "reset-size", "ideographic", "alphabetic", "hanging", "mathematical", "central", "middle", "text-after-edge", "text-before-edge" ],
		invalid_values: []
	},
	"fill": {
		domProp: "fill",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "blue" },
		initial_values: [ "black", "#000", "#000000", "rgb(0,0,0)", "rgba(0,0,0,1)" ],
		other_values: [ "green", "#fc3", "url('#myserver')", "url(foo.svg#myserver)", 'url("#myserver") green', "none", "currentColor" ],
		invalid_values: []
	},
	"fill-opacity": {
		domProp: "fillOpacity",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1", "2.8", "1.000" ],
		other_values: [ "0", "0.3", "-7.3" ],
		invalid_values: []
	},
	"fill-rule": {
		domProp: "fillRule",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "nonzero" ],
		other_values: [ "evenodd" ],
		invalid_values: []
	},
	"filter": {
		domProp: "filter",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#myfilt)" ],
		invalid_values: [ "url(#myfilt) none" ]
	},
	"flood-color": {
		domProp: "floodColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "blue" },
		initial_values: [ "black", "#000", "#000000", "rgb(0,0,0)", "rgba(0,0,0,1)" ],
		other_values: [ "green", "#fc3", "currentColor" ],
		invalid_values: [ "url('#myserver')", "url(foo.svg#myserver)", 'url("#myserver") green' ]
	},
	"flood-opacity": {
		domProp: "floodOpacity",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1", "2.8", "1.000" ],
		other_values: [ "0", "0.3", "-7.3" ],
		invalid_values: []
	},
	"image-rendering": {
		domProp: "imageRendering",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "optimizeSpeed", "optimizeQuality", "-moz-crisp-edges" ],
		invalid_values: []
	},
	"lighting-color": {
		domProp: "lightingColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "blue" },
		initial_values: [ "white", "#fff", "#ffffff", "rgb(255,255,255)", "rgba(255,255,255,1.0)", "rgba(255,255,255,42.0)" ],
		other_values: [ "green", "#fc3", "currentColor" ],
		invalid_values: [ "url('#myserver')", "url(foo.svg#myserver)", 'url("#myserver") green' ]
	},
	"marker": {
		domProp: "marker",
		inherited: true,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "marker-start", "marker-mid", "marker-end" ],
		initial_values: [ "none" ],
		other_values: [ "url(#mysym)" ],
		invalid_values: [ "none none", "url(#mysym) url(#mysym)", "none url(#mysym)", "url(#mysym) none" ]
	},
	"marker-end": {
		domProp: "markerEnd",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#mysym)" ],
		invalid_values: []
	},
	"marker-mid": {
		domProp: "markerMid",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#mysym)" ],
		invalid_values: []
	},
	"marker-start": {
		domProp: "markerStart",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#mysym)" ],
		invalid_values: []
	},
	"mask": {
		domProp: "mask",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(#mymask)" ],
		invalid_values: []
	},
	"shape-rendering": {
		domProp: "shapeRendering",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "optimizeSpeed", "crispEdges", "geometricPrecision" ],
		invalid_values: []
	},
	"stop-color": {
		domProp: "stopColor",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		prerequisites: { "color": "blue" },
		initial_values: [ "black", "#000", "#000000", "rgb(0,0,0)", "rgba(0,0,0,1)" ],
		other_values: [ "green", "#fc3", "currentColor" ],
		invalid_values: [ "url('#myserver')", "url(foo.svg#myserver)", 'url("#myserver") green' ]
	},
	"stop-opacity": {
		domProp: "stopOpacity",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1", "2.8", "1.000" ],
		other_values: [ "0", "0.3", "-7.3" ],
		invalid_values: []
	},
	"stroke": {
		domProp: "stroke",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "black", "#000", "#000000", "rgb(0,0,0)", "rgba(0,0,0,1)", "green", "#fc3", "url('#myserver')", "url(foo.svg#myserver)", 'url("#myserver") green', "currentColor" ],
		invalid_values: []
	},
	"stroke-dasharray": {
		domProp: "strokeDasharray",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "5px,3px,2px", "5px 3px 2px", "  5px ,3px	, 2px ", "1px", "5%", "3em" ],
		invalid_values: [ "-5px,3px,2px", "5px,3px,-2px" ]
	},
	"stroke-dashoffset": {
		domProp: "strokeDashoffset",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0", "-0px", "0em" ],
		other_values: [ "3px", "3%", "1em" ],
		invalid_values: []
	},
	"stroke-linecap": {
		domProp: "strokeLinecap",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "butt" ],
		other_values: [ "round", "square" ],
		invalid_values: []
	},
	"stroke-linejoin": {
		domProp: "strokeLinejoin",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "miter" ],
		other_values: [ "round", "bevel" ],
		invalid_values: []
	},
	"stroke-miterlimit": {
		domProp: "strokeMiterlimit",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "4" ],
		other_values: [ "1", "7", "5000", "1.1" ],
		invalid_values: [ "0.9", "0", "-1", "3px", "-0.3" ]
	},
	"stroke-opacity": {
		domProp: "strokeOpacity",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1", "2.8", "1.000" ],
		other_values: [ "0", "0.3", "-7.3" ],
		invalid_values: []
	},
	"stroke-width": {
		domProp: "strokeWidth",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "1px" ],
		other_values: [ "0", "0px", "-0em", "17px", "0.2em" ],
		invalid_values: [ "-0.1px", "-3px" ]
	},
	"text-anchor": {
		domProp: "textAnchor",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "start" ],
		other_values: [ "middle", "end" ],
		invalid_values: []
	},
	"text-rendering": {
		domProp: "textRendering",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "auto" ],
		other_values: [ "optimizeSpeed", "optimizeLegibility", "geometricPrecision" ],
		invalid_values: []
	}
}

function logical_box_prop_get_computed(cs, property)
{
	if (! /^-moz-/.test(property))
		throw "Unexpected property";
	property = property.substring(5);
	if (cs.getPropertyValue("direction") == "ltr")
		property = property.replace("-start", "-left").replace("-end", "-right");
	else
		property = property.replace("-start", "-right").replace("-end", "-left");
	return cs.getPropertyValue(property);
}

// Get the computed value for a property.  For shorthands, return the
// computed values of all the subproperties, delimited by " ; ".
function get_computed_value(cs, property)
{
	var info = gCSSProperties[property];
	if ("subproperties" in info) {
		var results = [];
		for (var idx in info.subproperties) {
			var subprop = info.subproperties[idx];
			results.push(get_computed_value(cs, subprop));
		}
		return results.join(" ; ");
	}
	if (info.get_computed)
		return info.get_computed(cs, property);
	return cs.getPropertyValue(property);
}
