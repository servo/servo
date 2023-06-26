commands.json
=============

:code:`commands.json` files define how subcommands are executed by the
:code:`./wpt` command. :code:`wpt` searches all command.json files under the top
directory and sets up subcommands from these JSON files. A typical commands.json
would look like the following::

  {
    "foo": {
      "path": "foo.py",
      "script": "run",
      "parser": "get_parser",
      "help": "Run foo"
    },
    "bar": {
      "path": "bar.py",
      "script": "run",
      "virtualenv": true,
      "requirements": [
        "requirements.txt"
      ]
    }
  }

Each key of the top level object defines a name of a subcommand, and its value
(a properties object) specifies how the subcommand is executed. Each properties
object must contain :code:`path` and :code:`script` fields and may contain
additional fields. All paths are relative to the commands.json.

:code:`path`
  The path to a Python script that implements the subcommand.

:code:`script`
  The name of a function that is used as the entry point of the subcommand.

:code:`parser`
  The name of a function that creates an argparse parser for the subcommand.

:code:`parse_known`
  When True, `parse_known_args() <https://docs.python.org/3/library/argparse.html#argparse.ArgumentParser.parse_known_args>`_
  is used instead of parse_args() for the subcommand. Default to False.

:code:`help`
  Brief description of the subcommand.

:code:`virtualenv`
  When True, the subcommand is executed with a virtualenv environment. Default
  to True.

:code:`requirements`
  A list of paths where each path specifies a requirements.txt. All requirements
  listed in these files are installed into the virtualenv environment before
  running the subcommand. :code:`virtualenv` must be true when this field is
  set.

:code:`conditional_requirements`
  A key-value object. Each key represents a condition, and value represents
  additional requirements when the condition is met. The requirements have the
  same format as :code:`requirements`. Currently "commandline_flag" is the only
  supported key. "commandline_flag" is used to specify requirements needed for a
  certain command line flag of the subcommand. For example, given the following
  commands.json::

    "baz": {
      "path": "baz.py",
      "script": "run",
      "virtualenv": true,
      "conditional_requirements": {
        "commandline_flag": {
          "enable_feature1": [
            "requirements_feature1.txt"
          ]
        }
      }
    }

  Requirements in :code:`requirements_features1.txt` are installed only when
  :code:`--enable-feature1` is specified to :code:`./wpt baz`.
