# Script tests

## Import attributes & JSON/CSS modules

The import attributes proposal changed the keyword from `assert` to `with`, after that it was already been implemented and shipped in some browsers. Thus, there are some implementations that only support the `assert` syntax and others that only support the `with` syntax.

For this reason, the import attributes, JSON modules and CSS modules are duplicated to use both keywords:
| `with` keyword        | `assert` keyword           |
|:----------------------|:---------------------------|
| `./import-attributes` | `./import-assertions`      |
| `./json-module`       | `./json-module-assertions` |
| `./css-module`        | `./css-module-assertions`  |

All changes in one folder should be reflected in the corresponding folder, because the two syntaxes have the same semantics.

The web compatibility of removing the `assert` keyword is being investigated. If it will be deemed feasible, it will be removed from the proposal and the `assert` tests can be deleted.
