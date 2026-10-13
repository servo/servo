# MathML AAM Reference: https://w3c.github.io/mathml-aam/#mathml-element-mappings
import pytest

TEST_DATA_ATTRIBUTES = {
    "a": {
        "html": """<math><a href="https://kuaishou.com" id="test"><mtext>Link</mtext></a></math>""",
    },
    "annotation": {
        "html": "<math><semantics><mn>2</mn><annotation id='test' encoding='application/x-tex' style='display: inline-block;'>\\text{two}</annotation></semantics></math>",
    },
    "annotation-xml": {
        "html": "<math><semantics><mn>2</mn><annotation-xml id='test' encoding='application/mathml-content+xml' style='display: inline-block;'><mtext>two</mtext></annotation-xml></semantics></math>",
    },
    "maction": {
        "html": "<math><maction id='test' actiontype='toggle'><mn>1</mn><mn>2</mn></maction></math>",
    },
    "math": {
        "html": "<math id='test'><mn>1</mn></math>",
    },
    "merror": {
        "html": "<math><merror id='test'><mtext>Divide by zero</mtext></merror></math>",
    },
    "mfrac": {
        "html": "<math><mfrac id='test'><mn id='num'>1</mn><mn id='den'>2</mn></mfrac></math>",
        "axapi": {"AXMathFractionNumerator": "num", "AXMathFractionDenominator": "den"}
    },
    "mi": {
        "html": "<math><mi id='test'>x</mi></math>",
    },
    # mmultiscripts object reference tested in mathml-aam/aamtests/attribute/mmultiscripts.py
    "mmultiscripts": {
        "html": "<math><mmultiscripts id='test'><mi id='base'>X</mi><mn id='sub'>1</mn><mn id='sup'>2</mn><mprescripts/><mn id='presub'>3</mn><mn id='presup'>4</mn></mmultiscripts></math>",
    },
    "mn": {
        "html": "<math><mn id='test'>2</mn></math>",
    },
    "mo": {
        "html": "<math><mo id='test'>+</mo></math>",
    },
    "mover": {
        "html": "<math><mover id='test'><mi id='base'>a</mi><mo id='over'>˙</mo></mover></math>",
        "axapi": {"AXMathBase": "base", "AXMathOver": "over"}
    },
    "mpadded": {
        "html": "<math><mpadded id='test' width='+10px'><mn>1</mn></mpadded></math>",
    },
    "mphantom": {
        "html": "<math><mphantom style='visibility: visible;' id='test'><mn>1</mn></mphantom></math>",
    },
    "mprescripts": {
        "html": "<math><mmultiscripts><mi>X</mi><mn>1</mn><mn>2</mn><mprescripts id='test'/><mn>3</mn><mn>4</mn></mmultiscripts></math>"
    },
    "mroot": {
        "html": "<math><mroot id='test'><mi id='rad'>x</mi><mn id='idx'>3</mn></mroot></math>",
        "axapi": {"AXMathRootRadicand": "rad", "AXMathRootIndex": "idx"}
    },
    "mrow": {
        "html": "<math><mrow id='test'><mn>1</mn><mo>+</mo><mn>1</mn></mrow></math>",
    },
    "ms": {
        "html": """<math><ms id="test">"string"</ms></math>""",
    },
    "mspace": {
        "html": "<math><mspace id='test' width='2em'/></math>"
    },
    "msqrt": {
        "html": "<math><msqrt id='test'><mi id='rad1'>x</mi></msqrt></math>",
        "axapi": {"AXMathRootRadicand": "rad1"}
    },
    "mstyle": {
        "html": "<math><mstyle id='test' mathcolor='blue'><mn>1</mn></mstyle></math>",
    },
    "msub": {
        "html": "<math><msub id='test'><mi id='base'>x</mi><mn id='sub'>1</mn></msub></math>",
        "axapi": {"AXMathBase": "base", "AXMathSubscript": "sub"}
    },
    "msubsup": {
        "html": "<math><msubsup id='test'><mi id='base'>x</mi><mn id='sub'>1</mn><mn id='sup'>2</mn></msubsup></math>",
        "axapi": {"AXMathBase": "base", "AXMathSubscript": "sub", "AXMathSuperscript": "sup"}
    },
    "msup": {
        "html": "<math><msup id='test'><mi id='base'>x</mi><mn id='sup'>2</mn></msup></math>",
        "axapi": {"AXMathBase": "base", "AXMathSuperscript": "sup"}
    },
    "mtable": {
        "html": "<math><mtable id='test'><mtr><mtd><mn>1</mn></mtd></mtr></mtable></math>",
        "atspi": {"interface": "AtkTable"}
    },
    "mtd": {
        "html": "<math><mtable><mtr><mtd id='test'><mn>1</mn></mtd></mtr></mtable></math>",
        "atspi": {"interface": "AtkTableCell"}
    },
    "mtext": {
        "html": "<math><mtext id='test'>hello</mtext></math>",
    },
    "mtr": {
        "html": "<math><mtable><mtr id='test'><mtd><mn>1</mn></mtd></mtr></mtable></math>",
    },
    "munder": {
        "html": "<math><munder id='test'><mi id='base'>x</mi><mo id='under'>_</mo></munder></math>",
        "axapi": {"AXMathBase": "base", "AXMathUnder": "under"}
    },
    "munderover": {
        "html": "<math><munderover id='test'><mi id='base'>x</mi><mo id='under'>_</mo><mo id='over'>˙</mo></munderover></math>",
        "axapi": {"AXMathBase": "base", "AXMathUnder": "under", "AXMathOver": "over"}
    },
    "none": {
        "html": "<math><mmultiscripts><mi>X</mi><mn>1</mn><none id='test'/></mmultiscripts></math>"
    },
    "semantics": {
        "html": "<math><semantics id='test'><mn>5</mn><annotation encoding='application/x-tex'>5</annotation></semantics></math>",
    }
}

@pytest.mark.parametrize("element_name", TEST_DATA_ATTRIBUTES.keys())
def test_atspi(atspi, session, inline, element_name):
    data = TEST_DATA_ATTRIBUTES[element_name]
    session.url = inline(data["html"])
    node = atspi.find_node("test", session.url)

    obj_attrs = atspi.Accessible.get_attributes(node)
    assert "tag" in obj_attrs, f"<{element_name}> object attributes missing 'tag' key. Found: {obj_attrs}"
    assert obj_attrs["tag"] == element_name, f"Expected tag '{element_name}', got '{obj_attrs['tag']}'"

    atspi_config = data.get("atspi", {})
    interface_name = atspi_config.get("interface")

    if interface_name:
        obj = atspi.get_interface(node, interface_name)
        assert obj is not None, f"{element_name} node must implement {interface_name} interface"


AX_ATTR_CASES = [name for name, d in TEST_DATA_ATTRIBUTES.items() if "axapi" in d]

@pytest.mark.parametrize("element_name", AX_ATTR_CASES)
def test_axapi(axapi, session, inline, element_name):
    data = TEST_DATA_ATTRIBUTES[element_name]
    session.url = inline(data["html"])
    node = axapi.find_node("test", session.url)

    expected_relationships = data["axapi"]

    for ax_attr_key, expected_dom_id in expected_relationships.items():
        attr_res = axapi.AXUIElementCopyAttributeValue(node, ax_attr_key, None)
        target_node = attr_res[1] if (attr_res and len(attr_res) > 1) else None

        assert target_node is not None, f"Relationship attribute '{ax_attr_key}' on <{element_name}> returned None."

        actual_dom_id = axapi.AXUIElementCopyAttributeValue(target_node, "AXDOMIdentifier", None)[1]
        assert actual_dom_id == expected_dom_id, f"Pointer '{ax_attr_key}' expected to target id '{expected_dom_id}', but got '{actual_dom_id}'"