#!/usr/bin/env python3
import os, shutil

target_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + "/compute-kind-widget-generated"

props_grouped = [
  [
    u"background-color",
    u"background-image",
  ],
  [
    u"background-attachment",
    u"background-position",
    u"background-clip",
    u"background-origin",
    u"background-size",
  ],
  [
    u"border-top-color",
    u"border-right-color",
    u"border-bottom-color",
    u"border-left-color",
  ],
  [
    u"border-top-style",
    u"border-right-style",
    u"border-bottom-style",
    u"border-left-style",
  ],
  [
    u"border-top-width",
    u"border-right-width",
    u"border-bottom-width",
    u"border-left-width",
  ],
  [
    u"border-block-start-color",
    u"border-block-end-color",
    u"border-inline-start-color",
    u"border-inline-end-color",
  ],
  [
    u"border-block-start-style",
    u"border-block-end-style",
    u"border-inline-start-style",
    u"border-inline-end-style",
  ],
  [
    u"border-block-start-width",
    u"border-block-end-width",
    u"border-inline-start-width",
    u"border-inline-end-width",
  ],
  [
    u"border-image-source",
    u"border-image-slice",
    u"border-image-width",
    u"border-image-outset",
    u"border-image-repeat",
  ],
  [
    u"border-top-left-radius",
    u"border-top-right-radius",
    u"border-bottom-right-radius",
    u"border-bottom-left-radius",
    u"border-start-start-radius",
    u"border-start-end-radius",
    u"border-end-start-radius",
    u"border-end-end-radius",
  ],
]

els = [
  [u'link', u'<a id="link">a</a>'],
  [u'button', u'<button id="button">button</button>'],
  [u'input-button', u'<input id="button-input" type="button" value="input-button">'],
  [u'input-submit', u'<input id="submit-input" type="submit" value="input-submit">'],
  [u'input-reset', u'<input id="reset-input" type="reset" value="input-reset">'],
  [u'input-text', u'<input id="text-input" type="text" value="input-text">'],
  [u'input-search-text', u'<input id="search-text-input" type="search" value="input-search-text">'],
  [u'input-search', u'<input id="search-input" type="search" value="input-search">'],
  [u'range', u'<input id="range-input" type="range">'],
  [u'checkbox-input', u'<input id="checkbox-input" type="checkbox">'],
  [u'radio-input', u'<input id="radio-input" type="radio">'],
  [u'color-input', u'<input id="color-input" type="color">'],
  [u'textarea', u'<textarea id="textarea">textarea</textarea>'],
  [u'select-listbox', u'<select multiple id="select-listbox"><option>select-listbox</option></select>'],
  [u'select-dropdown-box', u'<select id="select-dropdown-box"><option>select-dropdown-box</option></select>'],
  [u'select-menulist-button', u'<select id="select-menulist-button"><option>select-menulist-button</option></select>'],
  [u'meter', u'<meter id="meter" value=0.5></meter>'],
  [u'progress', u'<progress id="progress" value=0.5></progress>'],
]

all_els = ""
for el_id, el_markup in els:
    all_els += el_markup + "\n    "
all_els = all_els.rstrip()

template = u"""<!-- DO NOT EDIT. This file has been generated. Source:
    ../tools/build-compute-kind-widget-fallback-props.py
-->
<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Basic User Interface Test: Compute kind of widget: {props} maybe disables native appearance for {el_id}</title>
<link rel="help" href="https://drafts.csswg.org/css-ui-4/#appearance-disabling-properties">
<link rel="help" href="https://html.spec.whatwg.org/#widgets">
<meta name="assert" content="appropriate widget is used when props includes {props}.">
<link rel="match" href="../compute-kind-widget-fallback-{el_id}-ref.html">
<style>
    #container {{ width: 500px; }}
    /* NOTE: This rule is only used in the search-text-input tests: */
    #container > #search-text-input {{ appearance: textfield; }}
    /* NOTE: This rule is only used in the select-menulist-button tests: */
    #container > #select-menulist-button {{ appearance: none; appearance: menulist-button; }}
</style>

<div id="container">
    {el_markup}
</div>

<script>
// Set author-level CSS that matches UA style, but don't use the 'revert' value.
const elements = document.querySelectorAll('#container > *');
const props = "{props}".split(",");
for (const el of elements) {{
  for (const prop of props) {{
    el.style.setProperty(prop, getComputedStyle(el).getPropertyValue(prop));
  }}
}}
</script>
"""

# Generate tests

# wipe target_dir
if os.path.isdir(target_dir):
    shutil.rmtree(target_dir)

def write_file(path, content):
    path = os.path.join(target_dir, path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    file = open(os.path.join(target_dir, path), 'w')
    file.write(content)
    file.close()

def generate_tests(prop, el_id, el_markup):
    test = template.format(props=prop, el_id=el_id, el_markup=el_markup)
    write_file(f"kind-of-widget-fallback-{el_id}-{prop}-001.html", test)

def generate_grouped_tests(group):
    test = template.format(props=",".join(group), el_id="all-elements", el_markup=all_els)
    write_file(f"grouped-kind-of-widget-fallback-{group[0]}-001.html", test)

for group in props_grouped:
    generate_grouped_tests(group)
    for prop in group:
        for el_id, el_markup in els:
            generate_tests(prop, el_id, el_markup)
