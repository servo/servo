import json

import html5lib

def parse(path="html5ents.xml"):
    return html5lib.parse(open(path), treebuilder="lxml")

def entity_table(tree):
    return dict((entity_name("".join(tr[0].xpath(".//text()"))),
                 entity_characters(tr[1].text))
                for tr in tree.xpath("//h:tbody/h:tr",
                                     namespaces={"h":"http://www.w3.org/1999/xhtml"}))

def entity_name(inp):
    return inp.strip()

def entity_characters(inp):
    return "".join(codepoint_to_character(item)
                    for item in inp.split()
                    if item)

def codepoint_to_character(inp):
    return ("\U000"+inp[2:]).decode("unicode-escape")

def make_tests_json(entities):
    test_list = make_test_list(entities)
    tests_json = {"tests":
                      [make_test(*item) for item in test_list]
                  }
    return tests_json

def make_test(name, characters, good):
    return {
        "description":test_description(name, good),
        "input":"&%s"%name,
        "output":test_expected(name, characters, good)
        }

def test_description(name, good):
    with_semicolon = name.endswith(";")
    semicolon_text = {True:"with a semi-colon",
                      False:"without a semi-colon"}[with_semicolon]
    if good:
        text = "Named entity: %s %s"%(name, semicolon_text)
    else:
        text = "Bad named entity: %s %s"%(name, semicolon_text)
    return text

def test_expected(name, characters, good):
    rv = []
    if not good or not name.endswith(";"):
        rv.append("ParseError")
    rv.append(["Character", characters])
    return rv

def make_test_list(entities):
    tests = []
    for entity_name, characters in entities.items():
        if entity_name.endswith(";") and not subentity_exists(entity_name, entities):
            tests.append((entity_name[:-1], "&" + entity_name[:-1], False))
        tests.append((entity_name, characters, True))
    return sorted(tests)

def subentity_exists(entity_name, entities):
    for i in range(1, len(entity_name)):
        if entity_name[:-i] in entities:
            return True
    return False

def make_entities_code(entities):
    entities_text = "\n".join("    \"%s\": u\"%s\","%(
            name, entities[name].encode(
                "unicode-escape").replace("\"", "\\\""))
                              for name in sorted(entities.keys()))
    return """entities = {
%s
}"""%entities_text

def main():
    entities = entity_table(parse())
    tests_json = make_tests_json(entities)
    json.dump(tests_json, open("namedEntities.test", "w"), indent=4)
    code = make_entities_code(entities)
    open("entities_constants.py", "w").write(code)

if __name__ == "__main__":
    main()

