DEFAULT_CONTENT = "<div id='content'>Lorem ipsum dolor sit amet.</div>"

REFERENCE_CONTENT = f"<div id='outer'>{DEFAULT_CONTENT}</div>"
REFERENCE_STYLE = """
    <style>
      #outer {
        display: block;
        margin: 0;
        border: 0;
        width: 200px;
        height: 200px;
      }
      #content {
        display: block;
        margin: 0;
        border: 0;
        width: 100px;
        height: 100px;
        background: green;
      }
    </style>
"""

OUTER_IFRAME_STYLE = """
    <style>
      iframe {
        display: block;
        margin: 0;
        border: 0;
        width: 200px;
        height: 200px;
      }
    </style>
"""

INNER_IFRAME_STYLE = """
    <style>
      body {
        margin: 0;
      }
      div {
        display: block;
        margin: 0;
        border: 0;
        width: 100px;
        height: 100px;
        background: green;
      }
    </style>
"""
