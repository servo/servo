
.. _bash_completion:

Setting up bash completion
==========================

When using bash as your shell, ``pytest`` can use argcomplete
(https://argcomplete.readthedocs.org/) for auto-completion.
For this ``argcomplete`` needs to be installed **and** enabled.

Install argcomplete using::

        sudo pip install 'argcomplete>=0.5.7'

For global activation of all argcomplete enabled python applications run::

	sudo activate-global-python-argcomplete

For permanent (but not global) ``pytest`` activation, use::

        register-python-argcomplete py.test >> ~/.bashrc

For one-time activation of argcomplete for ``pytest`` only, use::

        eval "$(register-python-argcomplete py.test)"



