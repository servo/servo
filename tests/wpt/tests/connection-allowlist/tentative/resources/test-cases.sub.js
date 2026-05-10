const SUCCESS = true;
const FAILURE = false;

function get_test_cases(port) {
  return [
    { origin: "http://{{hosts[][]}}" + port },
    { origin: "http://{{hosts[][www]}}" + port },
    { origin: "http://{{hosts[][www1]}}" + port },
    { origin: "http://{{hosts[][www2]}}" + port },
    { origin: "http://{{hosts[][天気の良い日]}}" + port },
    { origin: "http://{{hosts[][élève]}}" + port },
    { origin: "http://{{hosts[alt][]}}" + port },
    { origin: "http://{{hosts[alt][www]}}" + port },
    { origin: "http://{{hosts[alt][www1]}}" + port },
    { origin: "http://{{hosts[alt][www2]}}" + port },
    { origin: "http://{{hosts[alt][天気の良い日]}}" + port },
    { origin: "http://{{hosts[alt][élève]}}" + port },
  ];
}

function get_default_expectations() {
  return [
    SUCCESS, // hosts[][]
    FAILURE, // hosts[][www]
    FAILURE, // hosts[][www1]
    FAILURE, // hosts[][www2]
    FAILURE, // hosts[][天気の良い日]
    FAILURE, // hosts[][élève]
    FAILURE, // hosts[alt][]
    FAILURE, // hosts[alt][www]
    FAILURE, // hosts[alt][www1]
    FAILURE, // hosts[alt][www2]
    FAILURE, // hosts[alt][天気の良い日]
    FAILURE, // hosts[alt][élève]
  ];
}

function get_wildcard_expectations() {
  return [
    SUCCESS, // hosts[][]
    FAILURE, // hosts[][www]
    FAILURE, // hosts[][www1]
    FAILURE, // hosts[][www2]
    FAILURE, // hosts[][天気の良い日]
    FAILURE, // hosts[][élève]
    FAILURE, // hosts[alt][]
    SUCCESS, // hosts[alt][www]
    SUCCESS, // hosts[alt][www1]
    SUCCESS, // hosts[alt][www2]
    SUCCESS, // hosts[alt][天気の良い日]
    SUCCESS, // hosts[alt][élève]
  ];
}
