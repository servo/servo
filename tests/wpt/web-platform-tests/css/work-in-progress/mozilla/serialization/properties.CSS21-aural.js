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

// Helper functions used to construct gCSSProperties.

var gCSSProperties = {
	"azimuth": {
		domProp: "azimuth",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "center", "0deg" ],
		other_values: [ "center behind", "behind far-right", "left-side", "73deg", "90.1deg", "0.1deg" ],
		invalid_values: [ "0deg behind", "behind 0deg", "90deg behind", "behind 90deg" ]
	},
	"cue": {
		domProp: "cue",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "cue-before", "cue-after" ],
		initial_values: [ "none", "none none" ],
		other_values: [ "url(404.wav)", "url(404.wav) none", "none url(404.wav)" ],
		invalid_values: []
	},
	"cue-after": {
		domProp: "cueAfter",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(404.wav)" ],
		invalid_values: []
	},
	"cue-before": {
		domProp: "cueBefore",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "url(404.wav)" ],
		invalid_values: []
	},
	"elevation": {
		domProp: "elevation",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "level", "0deg" ],
		other_values: [ "below", "above", "60deg", "higher", "lower", "-79deg", "0.33deg" ],
		invalid_values: []
	},
	"pause": {
		domProp: "pause",
		inherited: false,
		type: CSS_TYPE_TRUE_SHORTHAND,
		subproperties: [ "pause-before", "pause-after" ],
		initial_values: [ "0s", "0ms", "0s 0ms" ],
		other_values: [ "1s", "200ms", "-2s", "50%", "-10%", "10% 200ms", "-3s -5%" ],
		invalid_values: [ "0", "0px", "0 0", "0ms 0" ]
	},
	"pause-after": {
		domProp: "pauseAfter",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0s", "0ms" ],
		other_values: [ "1s", "200ms", "-2s", "50%", "-10%" ],
		invalid_values: [ "0", "0px" ]
	},
	"pause-before": {
		domProp: "pauseBefore",
		inherited: false,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "0s", "0ms" ],
		other_values: [ "1s", "200ms", "-2s", "50%", "-10%" ],
		invalid_values: [ "0", "0px" ]
	},
	"pitch": {
		domProp: "pitch",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "medium" ],
		other_values: [ "x-low", "low", "high", "x-high" ],
		invalid_values: []
	},
	"pitch-range": {
		domProp: "pitchRange",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "50", "50.0" ],
		other_values: [ "0", "100.0", "99.7", "47", "3.2" ],
		invalid_values: [" -0.01", "100.2", "108", "-3" ]
	},
	"richness": {
		domProp: "richness",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "50", "50.0" ],
		other_values: [ "0", "100.0", "99.7", "47", "3.2" ],
		invalid_values: [" -0.01", "100.2", "108", "-3" ]
	},
	"speak": {
		domProp: "speak",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "normal" ],
		other_values: [ "none", "spell-out" ],
		invalid_values: []
	},
	"speak-header": {
		domProp: "speakHeader",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "once" ],
		other_values: [ "always" ],
		invalid_values: []
	},
	"speak-numeral": {
		domProp: "speakNumeral",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "continuous" ],
		other_values: [ "digits" ],
		invalid_values: []
	},
	"speak-punctuation": {
		domProp: "speakPunctuation",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "none" ],
		other_values: [ "code" ],
		invalid_values: []
	},
	"speech-rate": {
		domProp: "speechRate",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "medium" ],
		other_values: [ "x-slow", "slow", "fast", "x-fast", "faster", "slower", "80", "500", "73.2" ],
		invalid_values: [
			// "0", "-80" // unclear
		]
	},
	"stress": {
		domProp: "stress",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "50", "50.0" ],
		other_values: [ "0", "100.0", "99.7", "47", "3.2" ],
		invalid_values: [" -0.01", "100.2", "108", "-3" ]
	},
	"voice-family": {
		domProp: "voiceFamily",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "male" ], /* arbitrary guess */
		other_values: [ "female", "child", "Bob, male", "Jane, Juliet, female" ],
		invalid_values: []
	},
	"volume": {
		domProp: "volume",
		inherited: true,
		type: CSS_TYPE_LONGHAND,
		initial_values: [ "50", "50.0", "medium" ],
		other_values: [ "0", "100.0", "99.7", "47", "3.2", "silent", "x-soft", "soft", "loud", "x-loud" ],
		invalid_values: [" -0.01", "100.2", "108", "-3" ]
	},
}
