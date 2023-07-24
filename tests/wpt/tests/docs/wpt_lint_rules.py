from docutils.parsers.rst import Directive, nodes
from docutils.utils import new_document
from recommonmark.parser import CommonMarkParser
import importlib
import textwrap

class WPTLintRules(Directive):
    """A docutils directive to generate documentation for the
    web-platform-test-test's linting tool from its source code. Requires a
    single argument: a Python module specifier for a file which declares
    linting rules."""
    has_content = True
    required_arguments = 1
    optional_arguments = 0
    _md_parser = CommonMarkParser()

    @staticmethod
    def _parse_markdown(markdown):
        WPTLintRules._md_parser.parse(markdown, new_document("<string>"))
        return WPTLintRules._md_parser.document.children[0]

    @property
    def module_specifier(self):
        return self.arguments[0]

    def _get_rules(self):
        try:
            module = importlib.import_module(self.module_specifier)
        except ImportError:
            raise ImportError(
                """wpt-lint-rules: unable to resolve the module at "{}".""".format(self.module_specifier)
            )

        for binding_name, value in module.__dict__.items():
            if hasattr(value, "__abstractmethods__") and len(value.__abstractmethods__):
                continue

            description = getattr(value, "description", None)
            name = getattr(value, "name", None)
            to_fix = getattr(value, "to_fix", None)

            if description is None:
                continue

            if to_fix is not None:
                to_fix = textwrap.dedent(to_fix)

            yield {
                "name": name,
                "description": textwrap.dedent(description),
                "to_fix": to_fix
            }


    def run(self):
        definition_list = nodes.definition_list()

        for rule in sorted(self._get_rules(), key=lambda rule: rule['name']):
            item = nodes.definition_list_item()
            definition = nodes.definition()
            term = nodes.term()
            item += term
            item += definition
            definition_list += item

            term += nodes.literal(text=rule["name"])
            definition += WPTLintRules._parse_markdown(rule["description"])

            if rule["to_fix"]:
                definition += nodes.strong(text="To fix:")
                definition += WPTLintRules._parse_markdown(rule["to_fix"])

        if len(definition_list.children) == 0:
            raise Exception(
                """wpt-lint-rules: no linting rules found at "{}".""".format(self.module_specifier)
            )

        return [definition_list]
