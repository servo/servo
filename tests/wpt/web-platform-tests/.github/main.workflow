workflow "Build & Release Manifest" {
  on = "push"
  resolves = ["manifest-build-and-tag"]
}

action "manifest-build-and-tag" {
  uses = "./tools/docker/github"
  runs = ["python", "tools/ci/manifest_build.py"]
  secrets = ["GITHUB_TOKEN"]
}

workflow "Build & Publish Documentation Website" {
  on = "push"
  resolves = ["website-build-and-publish"]
}

action "website-build-and-publish" {
  uses = "./tools/docker/documentation"
  runs = ["/bin/bash", "tools/ci/website_build.sh"]
  secrets = ["DEPLOY_TOKEN"]
}

workflow "Synchronize the Pull Request Preview" {
  on = "pull_request"
  resolves = "update-pr-preview"
}

action "update-pr-preview" {
  uses = "./tools/docker/github"
  runs = ["python", "tools/ci/update_pr_preview.py", "https://api.github.com"]
  secrets = ["GITHUB_TOKEN"]
}
