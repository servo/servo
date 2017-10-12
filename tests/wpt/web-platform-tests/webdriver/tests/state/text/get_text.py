import pytest
import uuid

from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline

# For failing tests, the Get Element Text end-point is used
# directly. In all other cases, the Element.text() function is used.

def test_getting_text_of_a_non_existant_element_is_an_error(session):
   session.url = inline("""<body>Hello world</body>""")
   id = uuid.uuid4()

   result = session.transport.send(
       "GET",
       "session/%s/element/%s/text" % (session.session_id, id))

   assert_error(result, "no such element")


def test_read_element_text(session):
    session.url = inline("""
        <body>
          Noise before <span id='id'>This has an ID</span>. Noise after
        </body>""")

    element = session.find.css("#id", all=False)

    assert element.text == "This has an ID"
