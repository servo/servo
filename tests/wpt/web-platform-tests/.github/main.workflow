workflow "Build & Release Manifest" {
  on = "push"
  resolves = ["manifest-build-and-tag"]
}

action "manifest-build-and-tag" {
  uses = "./tools/docker/github"
  runs = ["python", "tools/ci/manifest_build.py"]
  secrets = ["GITHUB_TOKEN"]
}
