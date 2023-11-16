# These functions are used by FLEDGE to determine the logic for the ad buyer.
# For our testing purposes, we only need the minimal amount of boilerplate
# code in place to allow them to be invoked properly and move the FLEDGE
# process along. The tests generally do usually not deal with reporting results,
# so we leave `reportWin` empty unless we need to call registerAdBeacon(). See
# `generateURNFromFledge` in "utils.js" to see how this file is used.

from wptserve.utils import isomorphic_decode

def main(request, response):
  # Set up response headers.
  headers = [
    ('Content-Type', 'Application/Javascript'),
    ('Ad-Auction-Allowed', 'true')
  ]

  # Parse URL params.
  requested_size = request.GET.first(b"requested-size", None)
  ad_with_size = request.GET.first(b"ad-with-size", None)
  automatic_beacon = request.GET.first(b"automatic-beacon", None)

  # Use URL params to modify Javascript.
  requested_size_check = ''
  if requested_size is not None:
    # request.GET stores URL keys and values in iso-8859-1 binary encoding. We
    # have to decode the values back to a string to parse width/height. Don't
    # bother sanitizing the size, because it is sanitized before auction logic
    # runs already.
    width, height = isomorphic_decode(requested_size).split('-')

    requested_size_check = (
      f'''
        if (!(browserSignals.requestedSize.width === '{width}') &&
             (browserSignals.requestedSize.height === '{height}')) {{
          throw new Error('requestedSize missing/incorrect in browserSignals');
        }}
      '''
    )

  render_obj = 'ad.renderURL'
  if ad_with_size is not None:
    render_obj = '{ url: ad.renderURL, width: "100px", height: "50px" }'

  component_render_obj = 'component.renderURL'
  if ad_with_size is not None:
    component_render_obj = (
      '''{
          url: component.renderURL,
          width: "100px",
          height: "50px"
         }
      '''
    )

  register_ad_beacon = ''
  if automatic_beacon is not None:
    register_ad_beacon = (
    '''registerAdBeacon({
        'reserved.top_navigation_start':
        browserSignals.interestGroupOwner +
        '/fenced-frame/resources/automatic-beacon-store.py',
        'reserved.top_navigation_commit':
        browserSignals.interestGroupOwner +
        '/fenced-frame/resources/automatic-beacon-store.py',
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
        {requested_size_check}
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
