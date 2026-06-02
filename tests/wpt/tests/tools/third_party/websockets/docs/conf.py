# Configuration file for the Sphinx documentation builder.
#
# This file only contains a selection of the most common options. For a full
# list see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import datetime
import importlib
import inspect
import os
import subprocess
import sys

# -- Path setup --------------------------------------------------------------

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
sys.path.insert(0, os.path.join(os.path.abspath(".."), "src"))


# -- Project information -----------------------------------------------------

project = "websockets"
copyright = f"2013-{datetime.date.today().year}, Aymeric Augustin and contributors"
author = "Aymeric Augustin"

from websockets.version import tag as version, version as release


# -- General configuration ---------------------------------------------------

nitpicky = True

nitpick_ignore = [
    # topics/design.rst discusses undocumented APIs
    ("py:meth", "client.WebSocketClientProtocol.handshake"),
    ("py:meth", "server.WebSocketServerProtocol.handshake"),
    ("py:attr", "legacy.protocol.WebSocketCommonProtocol.is_client"),
    ("py:attr", "legacy.protocol.WebSocketCommonProtocol.messages"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.close_connection"),
    ("py:attr", "legacy.protocol.WebSocketCommonProtocol.close_connection_task"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.keepalive_ping"),
    ("py:attr", "legacy.protocol.WebSocketCommonProtocol.keepalive_ping_task"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.transfer_data"),
    ("py:attr", "legacy.protocol.WebSocketCommonProtocol.transfer_data_task"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.connection_open"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.ensure_open"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.fail_connection"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.connection_lost"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.read_message"),
    ("py:meth", "legacy.protocol.WebSocketCommonProtocol.write_frame"),
]

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.intersphinx",
    "sphinx.ext.linkcode",
    "sphinx.ext.napoleon",
    "sphinx_copybutton",
    "sphinx_inline_tabs",
    "sphinxcontrib.spelling",
    "sphinxcontrib_trio",
    "sphinxext.opengraph",
]
# It is currently inconvenient to install PyEnchant on Apple Silicon.
try:
    import sphinxcontrib.spelling
except ImportError:
    extensions.remove("sphinxcontrib.spelling")

autodoc_typehints = "description"

autodoc_typehints_description_target = "documented"

# Workaround for https://github.com/sphinx-doc/sphinx/issues/9560
from sphinx.domains.python import PythonDomain

assert PythonDomain.object_types["data"].roles == ("data", "obj")
PythonDomain.object_types["data"].roles = ("data", "class", "obj")

intersphinx_mapping = {"python": ("https://docs.python.org/3", None)}

spelling_show_suggestions = True

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# Configure viewcode extension.
from websockets.version import commit

code_url = f"https://github.com/python-websockets/websockets/blob/{commit}"

def linkcode_resolve(domain, info):
    # Non-linkable objects from the starter kit in the tutorial.
    if domain == "js" or info["module"] == "connect4":
        return

    assert domain == "py", "expected only Python objects"

    mod = importlib.import_module(info["module"])
    if "." in info["fullname"]:
        objname, attrname = info["fullname"].split(".")
        obj = getattr(mod, objname)
        try:
            # object is a method of a class
            obj = getattr(obj, attrname)
        except AttributeError:
            # object is an attribute of a class
            return None
    else:
        obj = getattr(mod, info["fullname"])

    try:
        file = inspect.getsourcefile(obj)
        lines = inspect.getsourcelines(obj)
    except TypeError:
        # e.g. object is a typing.Union
        return None
    file = os.path.relpath(file, os.path.abspath(".."))
    if not file.startswith("src/websockets"):
        # e.g. object is a typing.NewType
        return None
    start, end = lines[1], lines[1] + len(lines[0]) - 1

    return f"{code_url}/{file}#L{start}-L{end}"

# Configure opengraph extension

# Social cards don't support the SVG logo. Also, the text preview looks bad.
ogp_social_cards = {"enable": False}


# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
html_theme = "furo"

html_theme_options = {
    "light_css_variables": {
        "color-brand-primary": "#306998",  # blue from logo
        "color-brand-content": "#0b487a",  # blue more saturated and less dark
    },
    "dark_css_variables": {
        "color-brand-primary": "#ffd43bcc",  # yellow from logo, more muted than content
        "color-brand-content": "#ffd43bd9",  # yellow from logo, transparent like text
    },
    "sidebar_hide_name": True,
}

html_logo = "_static/websockets.svg"

html_favicon = "_static/favicon.ico"

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ["_static"]

html_copy_source = False

html_show_sphinx = False
