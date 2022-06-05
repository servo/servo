def main(request, response):
  if b"cookie" in request.cookies:
    return (500, [], "Cookie should not be sent to this endpoint")
  return """
{
  "privacy_policy_url": "https://privacypolicy.com"
}
"""
