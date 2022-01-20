#!/usr/bin/env python3
import os, shutil

target_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__))) + "/compute-kind-widget-generated"

props = [
  u"background-color",
  u"border-top-color",
  u"border-top-style",
  u"border-top-width",
  u"border-right-color",
  u"border-right-style",
  u"border-right-width",
  u"border-bottom-color",
  u"border-bottom-style",
  u"border-bottom-width",
  u"border-left-color",
  u"border-left-style",
  u"border-left-width",
  u"border-block-start-color",
  u"border-block-end-color",
  u"border-inline-start-color",
  u"border-inline-end-color",
  u"border-block-start-style",
  u"border-block-end-style",
  u"border-inline-start-style",
  u"border-inline-end-style",
  u"border-block-start-width",
  u"border-block-end-width",
  u"border-inline-start-width",
  u"border-inline-end-width",
  u"background-image",
  u"background-attachment",
  u"background-position",
  u"background-clip",
  u"background-origin",
  u"background-size",
  u"border-image-source",
  u"border-image-slice",
  u"border-image-width",
  u"border-image-outset",
  u"border-image-repeat",
  u"border-top-left-radius",
  u"border-top-right-radius",
  u"border-bottom-right-radius",
  u"border-bottom-left-radius",
  u"border-start-start-radius",
  u"border-start-end-radius",
  u"border-end-start-radius",
  u"border-end-end-radius",
]

template = u"""<!-- DO NOT EDIT. This file has been generated. Source:
    ./tools/build-compute-kind-widget-fallback-props.py
-->
<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Basic User Interface Test: Compute kind of widget: {prop} disables native appearance for widgets</title>
<link rel="help" href="https://drafts.csswg.org/css-ui-4/#computing-kind-widget">
<meta name="assert" content="appropriate widget is returned when authorProps includes {prop}.">
<link rel="match" href="../compute-kind-widget-fallback-ref.html">
<style>
    #container {{ width: 500px; }}
    #container > #search-text-input {{ appearance: textfield; }}
    #container > #select-menulist-button {{ appearance: none; appearance: menulist-button; }}
</style>

<div id="container">
    <a>a</a>
    <button id="button">button</button>
    <input id="button-input" type="button" value="input-button">
    <input id="submit-input" type="submit" value="input-submit">
    <input id="reset-input" type="reset" value="input-reset">

    <input id="text-input" type="text" value="input-text">
    <input id="search-text-input" type="search" value="input-search-text">
    <input id="search-input" type="search" value="input-search">

    <input id="range-input" type="range">
    <input id="checkbox-input" type="checkbox">
    <input id="radio-input" type="radio">
    <input id="color-input" type="color">

    <textarea id="textarea">textarea</textarea>
    <select multiple id="select-listbox"><option>select-listbox</option></select>
    <select id="select-dropdown-box"><option>select-dropdown-box</option></select>
    <select id="select-menulist-button"><option>select-menulist-button</option></select>
    <meter id="meter" value=0.5></meter>
    <progress id="progress" value=0.5></progress>
</div>

<script>
// Set author-level CSS that matches UA style, but don't use the 'revert' value.
const elements = document.querySelectorAll('#container > *');
const prop = "{prop}";
for (const el of elements) {{
  el.style.setProperty(prop, getComputedStyle(el).getPropertyValue(prop));
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

def generate_tests(prop):
    test = template.format(prop=prop)
    write_file(f"kind-of-widget-fallback-{prop}-001.html", test)

for prop in props:
    generate_tests(prop)
