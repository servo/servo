# *attrs*: Classes Without Boilerplate

Release **{sub-ref}`release`**  ([What's new?](changelog.md))

```{include} ../README.md
:start-after: 'teaser-begin -->'
:end-before: '<!-- teaser-end'
```


## Getting Started

*attrs* is a Python-only package [hosted on PyPI](https://pypi.org/project/attrs/).
The recommended installation method is [pip](https://pip.pypa.io/en/stable/)-installing into a [virtualenv](https://hynek.me/articles/virtualenv-lives/):

```console
$ python -Im pip install attrs
```

The next steps will get you up and running in no time:

- {doc}`overview` will show you a simple example of *attrs* in action and introduce you to its philosophy.
  Afterwards, you can start writing your own classes and understand what drives *attrs*'s design.
- {doc}`examples` will give you a comprehensive tour of *attrs*'s features.
  After reading, you will know about our advanced features and how to use them.
- {doc}`why` gives you a rundown of potential alternatives and why we think *attrs* is still worthwhile -- depending on *your* needs even superior.
- If at any point you get confused by some terminology, please check out our {doc}`glossary`.

If you need any help while getting started, feel free to use the `python-attrs` tag on [Stack Overflow](https://stackoverflow.com/questions/tagged/python-attrs) and someone will surely help you out!


## Day-to-Day Usage

- {doc}`types` help you to write *correct* and *self-documenting* code.
  *attrs* has first class support for them, yet keeps them optional if you’re not convinced!
- Instance initialization is one of *attrs* key feature areas.
  Our goal is to relieve you from writing as much code as possible.
  {doc}`init` gives you an overview what *attrs* has to offer and explains some related philosophies we believe in.
- Comparing and ordering objects is a common task.
  {doc}`comparison` shows you how *attrs* helps you with that and how you can customize it.
- If you want to put objects into sets or use them as keys in dictionaries, they have to be hashable.
  The simplest way to do that is to use frozen classes, but the topic is more complex than it seems and {doc}`hashing` will give you a primer on what to look out for.
- Once you're comfortable with the concepts, our {doc}`api` contains all information you need to use *attrs* to its fullest.
- *attrs* is built for extension from the ground up.
  {doc}`extending` will show you the affordances it offers and how to make it a building block of your own projects.
- Finally, if you're confused by all the `attr.s`, `attr.ib`, `attrs`, `attrib`, `define`, `frozen`, and `field`, head over to {doc}`names` for a very short explanation, and optionally a quick history lesson.


## *attrs* for Enterprise

```{include} ../README.md
:start-after: '### *attrs* for Enterprise'
```

---

## Full Table of Contents

```{toctree}
:maxdepth: 2

overview
why
examples
types
init
comparison
hashing
api
api-attr
extending
how-does-it-work
glossary
```

```{toctree}
:caption: Meta
:maxdepth: 1

names
license
changelog
Third-party Extentions <https://github.com/python-attrs/attrs/wiki/Extensions-to-attrs>
PyPI <https://pypi.org/project/attrs/>
Contributing <https://github.com/python-attrs/attrs/blob/main/.github/CONTRIBUTING.md>
Funding <https://hynek.me/say-thanks/>
```

[Full Index](genindex)
