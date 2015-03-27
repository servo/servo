def main(request, response):
    code = """
test(function() {
  assert_equals(self.%s, undefined);
});
""" % request.GET["varName"]

    return ([("Content-Type", "text/javascript")], code)
