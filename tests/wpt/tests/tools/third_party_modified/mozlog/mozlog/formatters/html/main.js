/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

function toArray(iter) {
  if (iter === null) {
    return null;
  }
  return Array.prototype.slice.call(iter);
}

function find(selector, elem) {
  if (!elem) {
    elem = document;
  }
  return elem.querySelector(selector);
}

function find_all(selector, elem) {
  if (!elem) {
    elem = document;
  }
  return toArray(elem.querySelectorAll(selector));
}

addEventListener("DOMContentLoaded", function () {
  reset_sort_headers();

  split_debug_onto_two_rows();

  find_all(".col-links a.screenshot").forEach(function (elem) {
    elem.addEventListener("click", function (event) {
      var node = elem;
      while (node && !node.classList.contains("results-table-row")) {
        node = node.parentNode;
      }
      if (node != null) {
        if (node.nextSibling && node.nextSibling.classList.contains("debug")) {
          var href = find(".screenshot img", node.nextSibling).src;
          window.open(href);
        }
      }
      event.preventDefault();
    });
  });

  find_all(".screenshot a").forEach(function (elem) {
    elem.addEventListener("click", function (event) {
      window.open(find("img", elem).getAttribute("src"));
      event.preventDefault();
    });
  });

  find_all(".sortable").forEach(function (elem) {
    elem.addEventListener("click", function (event) {
      toggle_sort_states(elem);
      var colIndex = toArray(elem.parentNode.childNodes).indexOf(elem);
      var key = elem.classList.contains("numeric") ? key_num : key_alpha;
      sort_table(elem, key(colIndex));
    });
  });
});

function sort_table(clicked, key_func) {
  one_row_for_data();
  var rows = find_all(".results-table-row");
  var reversed = !clicked.classList.contains("asc");

  var sorted_rows = sort(rows, key_func, reversed);

  var parent = document.getElementById("results-table-body");
  sorted_rows.forEach(function (elem) {
    parent.appendChild(elem);
  });

  split_debug_onto_two_rows();
}

function sort(items, key_func, reversed) {
  var sort_array = items.map(function (item, i) {
    return [key_func(item), i];
  });
  var multiplier = reversed ? -1 : 1;

  sort_array.sort(function (a, b) {
    var key_a = a[0];
    var key_b = b[0];
    return multiplier * (key_a >= key_b ? 1 : -1);
  });

  return sort_array.map(function (item) {
    var index = item[1];
    return items[index];
  });
}

function key_alpha(col_index) {
  return function (elem) {
    return elem.childNodes[col_index].firstChild.data.toLowerCase();
  };
}

function key_num(col_index) {
  return function (elem) {
    return parseFloat(elem.childNodes[col_index].firstChild.data);
  };
}

function reset_sort_headers() {
  find_all(".sort-icon").forEach(function (elem) {
    elem.remove();
  });
  find_all(".sortable").forEach(function (elem) {
    var icon = document.createElement("div");
    icon.className = "sort-icon";
    icon.textContent = "vvv";
    elem.insertBefore(icon, elem.firstChild);
    elem.classList.remove("desc", "active");
    elem.classList.add("asc", "inactive");
  });
}

function toggle_sort_states(elem) {
  // if active, toggle between asc and desc
  if (elem.classList.contains("active")) {
    elem.classList.toggle("asc");
    elem.classList.toggle("desc");
  }

  // if inactive, reset all other functions and add ascending active
  if (elem.classList.contains("inactive")) {
    reset_sort_headers();
    elem.classList.remove("inactive");
    elem.classList.add("active");
  }
}

function split_debug_onto_two_rows() {
  find_all("tr.results-table-row").forEach(function (elem) {
    var new_row = document.createElement("tr");
    new_row.className = "debug";
    elem.parentNode.insertBefore(new_row, elem.nextSibling);
    find_all(".debug", elem).forEach(function (td_elem) {
      if (find(".log", td_elem)) {
        new_row.appendChild(td_elem);
        td_elem.colSpan = 5;
      } else {
        td_elem.remove();
      }
    });
  });
}

function one_row_for_data() {
  find_all("tr.results-table-row").forEach(function (elem) {
    if (elem.nextSibling.classList.contains("debug")) {
      toArray(elem.nextSibling.childNodes).forEach(function (td_elem) {
        elem.appendChild(td_elem);
      });
    } else {
      var new_td = document.createElement("td");
      new_td.className = "debug";
      elem.appendChild(new_td);
    }
  });
}
