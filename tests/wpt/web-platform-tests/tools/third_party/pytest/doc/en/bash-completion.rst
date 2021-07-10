
.. _bash_completion:

Setting up bash completion
==========================

When using bash as your shell, ``pytest`` can use argcomplete
(https://argcomplete.readthedocs.io/) for auto-completion.
For this ``argcomplete`` needs to be installed **and** enabled.

Install argcomplete using:

.. code-block:: bash

    sudo pip install 'argcomplete>=0.5.7'

For global activation of all argcomplete enabled python applications run:

.. code-block:: bash

    sudo activate-global-python-argcomplete

For permanent (but not global) ``pytest`` activation, use:

.. code-block:: bash

    register-python-argcomplete pytest >> ~/.bashrc

For one-time activation of argcomplete for ``pytest`` only, use:

.. code-block:: bash

    eval "$(register-python-argcomplete pytest)"
