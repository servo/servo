(function() {
  var site_nav = document.querySelector(".wpt-site-nav");
  var trigger = document.querySelector(".wpt-site-nav .trigger");

  var show = function(e) {
    trigger.setAttribute("aria-hidden", "false");
  };

  var hide_if_relatedTarget_elsewhere = function(e) {
    if (!site_nav.contains(e.relatedTarget)) {
      trigger.setAttribute("aria-hidden", "true");
    }
  };

  site_nav.addEventListener("focus", show, false);
  site_nav.addEventListener("blur", hide_if_relatedTarget_elsewhere, true);

  site_nav.addEventListener("mouseenter", show, false);
  site_nav.addEventListener("mouseleave", hide_if_relatedTarget_elsewhere, false);
})();
