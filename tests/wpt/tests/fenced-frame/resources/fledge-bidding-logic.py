# These functions are used by FLEDGE to determine the logic for the ad buyer.
# For our testing purposes, we only need the minimal amount of boilerplate
# code in place to allow them to be invoked properly and move the FLEDGE
# process along. The tests generally do usually not deal with reporting results,
# so we leave `reportWin` empty unless we need to call registerAdBeacon(). See
# `generateURNFromFledge` in "utils.js" to see how this file is used.

def main(request, response):
  # Set up response headers.
  headers = [
    ('Content-Type', 'Application/Javascript'),
    ('X-Allow-FLEDGE', 'true')
  ]

  # Parse URL params.
  ad_with_size = request.GET.first(b"ad-with-size", None)
  automatic_beacon = request.GET.first(b"automatic-beacon", None)

  # Use URL params to modify JS
  render_obj = 'ad.renderUrl'
  if ad_with_size is not None:
    render_obj = '{ url: ad.renderUrl, width: "100px", height: "50px" }'

  component_render_obj = 'component.renderUrl'
  if ad_with_size is not None:
    component_render_obj = (
      '''{
          url: component.renderUrl,
          width: "100px",
          height: "50px"
         }
      '''
    )

  register_ad_beacon = ''
  if automatic_beacon is not None:
    register_ad_beacon = (
    '''registerAdBeacon({
        'reserved.top_navigation':
        browserSignals.interestGroupOwner +
        '/fenced-frame/resources/automatic-beacon-store.py'
      });
    '''
  )

  # Generate Javascript.
  # Note: Python fstrings use double-brackets ( {{, }} ) to insert bracket
  # literals instead of substitution sequences.
  generate_bid = (
    f'''function generateBid(
      interestGroup,
      auctionSignals,
      perBuyerSignals,
      trustedBiddingSignals,
      browserSignals) {{
        const ad = interestGroup.ads[0];

        // `auctionSignals` controls whether or not component auctions are
        // allowed.
        let allowComponentAuction = (typeof auctionSignals === 'string' &&
          auctionSignals.includes('bidderAllowsComponentAuction'));

        let result = {{
          'ad': ad,
          'bid': 1,
          'render': {render_obj},
          'allowComponentAuction': allowComponentAuction
        }};
        if (interestGroup.adComponents && interestGroup.adComponents.length > 0)
          result.adComponents = interestGroup.adComponents.map((component) => {{
            return {component_render_obj};
          }});
        return result;
      }}
    '''
  )

  report_win = (
    f'''function reportWin(
      auctionSignals,
      perBuyerSignals,
      sellerSignals,
      browserSignals) {{
        {register_ad_beacon}
        return;
      }}
    '''
  )

  content = f'{generate_bid}\n{report_win}'

  return (headers, content)
