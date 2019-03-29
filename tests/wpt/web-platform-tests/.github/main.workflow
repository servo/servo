workflow "Build & Release Manifest" {
  on = "push"
  resolves = ["tag-master"]
}

action "build-manifest" {
  uses = "./tools/docker/github"
  runs = ["bash", "-c", "tools/ci/action_manifest_build.sh"]
}

action "tag-master" {
  needs = "build-manifest"
  uses = "./tools/docker/github"
  runs = ["python", "tools/ci/tag_master.py"]
  secrets = ["GITHUB_TOKEN"]
}
