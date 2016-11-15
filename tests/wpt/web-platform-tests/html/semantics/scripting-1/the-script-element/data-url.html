<!doctype html>
<meta charset="utf-8">
<title>data URL and scripts</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
  setup({allow_uncaught_exception:true})
  async_test(t => {
    var counter = 1
    window.onerror = t.step_func((message, x, xx, xxx, e) => {
      assert_not_equals(message, "Script error.") // Cannot be "muted" as data URLs are same-origin
      assert_equals(typeof e, "number")
      assert_equals(e, counter)
      if (counter == 3) {
        t.done()
      }
      counter++
    })
  })
</script>
<script src="data:,throw 1"></script>
<script src="data:,throw 2" crossorigin></script>
<script src="data:,throw 3" crossorigin=use-credentials></script>
