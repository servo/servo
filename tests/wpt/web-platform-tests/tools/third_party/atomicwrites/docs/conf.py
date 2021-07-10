#!/usr/bin/env python

import os
import sys
import pkg_resources

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.intersphinx',
    'sphinx.ext.viewcode',
]

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# The suffix of source filenames.
source_suffix = '.rst'

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = 'atomicwrites'
copyright = '2015, Markus Unterwaditzer'

try:
    # The full version, including alpha/beta/rc tags.
    release = pkg_resources.require('atomicwrites')[0].version
except pkg_resources.DistributionNotFound:
    print('To build the documentation, the distribution information of '
          'atomicwrites has to be available. Run "setup.py develop" to do '
          'this.')
    sys.exit(1)

version = '.'.join(release.split('.')[:2])  # The short X.Y version.

on_rtd = os.environ.get('READTHEDOCS', None) == 'True'

try:
    import sphinx_rtd_theme
    html_theme = 'sphinx_rtd_theme'
    html_theme_path = [sphinx_rtd_theme.get_html_theme_path()]
except ImportError:
    html_theme = 'default'
    if not on_rtd:
        print('-' * 74)
        print('Warning: sphinx-rtd-theme not installed, building with default '
              'theme.')
        print('-' * 74)


# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
exclude_patterns = ['_build']

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']


# Output file base name for HTML help builder.
htmlhelp_basename = 'atomicwritesdoc'


# -- Options for LaTeX output ---------------------------------------------

latex_elements = {}

# Grouping the document tree into LaTeX files. List of tuples
# (source start file, target name, title,
#  author, documentclass [howto, manual, or own class]).
latex_documents = [
  ('index', 'atomicwrites.tex', 'atomicwrites Documentation',
   'Markus Unterwaditzer', 'manual'),
]

# One entry per manual page. List of tuples
# (source start file, name, description, authors, manual section).
man_pages = [
    ('index', 'atomicwrites', 'atomicwrites Documentation',
     ['Markus Unterwaditzer'], 1)
]

# Grouping the document tree into Texinfo files. List of tuples
# (source start file, target name, title, author,
#  dir menu entry, description, category)
texinfo_documents = [
  ('index', 'atomicwrites', 'atomicwrites Documentation',
   'Markus Unterwaditzer', 'atomicwrites', 'One line description of project.',
   'Miscellaneous'),
]

# Bibliographic Dublin Core info.
epub_title = 'atomicwrites'
epub_author = 'Markus Unterwaditzer'
epub_publisher = 'Markus Unterwaditzer'
epub_copyright = '2015, Markus Unterwaditzer'

# A list of files that should not be packed into the epub file.
epub_exclude_files = ['search.html']

# Example configuration for intersphinx: refer to the Python standard library.
intersphinx_mapping = {'http://docs.python.org/': None}
