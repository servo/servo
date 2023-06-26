# mypy: allow-untyped-defs

from tools.wpt import markdown

def test_format_comment_title():
    assert '# Browser #' == markdown.format_comment_title("browser")
    assert '# Browser (channel) #' == markdown.format_comment_title("browser:channel")

def test_markdown_adjust():
    assert '\\t' == markdown.markdown_adjust('\t')
    assert '\\r' == markdown.markdown_adjust('\r')
    assert '\\n' == markdown.markdown_adjust('\n')
    assert '' == markdown.markdown_adjust('`')
    assert '\\|' == markdown.markdown_adjust('|')
    assert '\\t\\r\\n\\|' == markdown.markdown_adjust('\t\r\n`|')

result = ''
def log(text):
    global result
    result += text

def test_table():
    global result
    headings = ['h1','h2']
    data = [['0', '1']]
    markdown.table(headings, data, log)
    assert ("| h1 | h2 |"
            "|----|----|"
            "| 0  | 1  |") == result

    result = ''
    data.append(['aaa', 'bb'])
    markdown.table(headings, data, log)
    assert ("|  h1 | h2 |"
            "|-----|----|"
            "| 0   | 1  |"
            "| aaa | bb |") == result
