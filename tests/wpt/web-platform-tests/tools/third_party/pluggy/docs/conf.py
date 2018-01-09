# -*- coding: utf-8 -*-
import pkg_resources


extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.doctest',
    'sphinx.ext.intersphinx',
    'sphinx.ext.coverage',
    'sphinx.ext.viewcode',
]

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

source_suffix = '.rst'

# The master toctree document.
master_doc = 'index'

# General information about the project.

dist = pkg_resources.get_distribution('pluggy')
project = dist.project_name
copyright = u'2016, Holger Krekel'
author = 'Holger Krekel'

release = dist.version
# The short X.Y version.
version = u'.'.join(dist.version.split('.')[:2])


language = None

pygments_style = 'sphinx'
html_logo = '_static/img/plug.png'
html_theme = 'alabaster'
html_theme_options = {
    # 'logo': 'img/plug.png',
    # 'logo_name': 'true',
    'description': 'The `pytest` plugin system',
    'github_user': 'pytest-dev',
    'github_repo': 'pluggy',
    'github_button': 'true',
    'github_banner': 'true',
    'page_width': '1080px',
    'fixed_sidebar': 'false',
}
html_static_path = ['_static']

# One entry per manual page. List of tuples
# (source start file, name, description, authors, manual section).
man_pages = [
    (master_doc, 'pluggy', u'pluggy Documentation',
     [author], 1)
]


# -- Options for Texinfo output -------------------------------------------

# Grouping the document tree into Texinfo files. List of tuples
# (source start file, target name, title, author,
#  dir menu entry, description, category)
texinfo_documents = [
    (master_doc, 'pluggy', u'pluggy Documentation',
     author, 'pluggy', 'One line description of project.',
     'Miscellaneous'),
]

# Example configuration for intersphinx: refer to the Python standard library.
intersphinx_mapping = {'https://docs.python.org/': None}
