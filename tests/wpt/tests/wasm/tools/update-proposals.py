#!/usr/bin/env python3
import os
import json
import subprocess
import shutil
import sys

def run_cmd(cmd, cwd=None):
    print(f"Running: {' '.join(cmd)} in {cwd or os.getcwd()}")
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=True).stdout


def main():
    wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    proposals_json_path = os.path.join(wpt_root, "wasm", "proposals.json")

    if not os.path.exists(proposals_json_path):
        raise FileNotFoundError(f"Proposals config file not found at {proposals_json_path}")

    with open(proposals_json_path, "r") as f:
        proposals = json.load(f)

    if not isinstance(proposals, list):
        raise TypeError("Proposals config must be a list of strings.")

    if not proposals:
        print("Proposals list is empty.")
        return

    # 1. Clear out target proposals directory entirely to prune removed proposals
    proposals_dir = os.path.join(wpt_root, "wasm", "proposals")
    if os.path.exists(proposals_dir):
        shutil.rmtree(proposals_dir)
    os.makedirs(proposals_dir, exist_ok=True)

    tmp_dir = os.path.join(os.getcwd(), "tmp_proposals")
    if os.path.exists(tmp_dir):
        shutil.rmtree(tmp_dir)
    os.makedirs(tmp_dir)

    updated_proposals = []
    merged_proposals = []

    try:
        for name in proposals:
            if not name or not isinstance(name, str):
                print(f"Invalid proposal name: {name}")
                continue

            print(f"\n--- Processing proposal: {name} ---")
            proposal_dir = os.path.join(tmp_dir, name)

            # 1. Clone proposal repo (start with main branch)
            run_cmd(["git", "clone", "--branch", "main", f"https://github.com/WebAssembly/{name}.git", proposal_dir])

            # 2. Add upstream remote to find fork point
            run_cmd(["git", "remote", "add", "upstream", "https://github.com/WebAssembly/spec.git"], cwd=proposal_dir)
            run_cmd(["git", "fetch", "upstream", "main"], cwd=proposal_dir)

            # 3. Find the merge base (fork point) commit
            try:
                base_commit = run_cmd(["git", "merge-base", "upstream/main", "HEAD"], cwd=proposal_dir).strip()
                print(f"Fork point commit: {base_commit}")
            except subprocess.CalledProcessError as e:
                print(f"Failed to find merge-base against upstream/main: {e}")
                print("Falling back to diffing against upstream/main directly.")
                base_commit = "upstream/main"

            # 4. Find changed or added files under test/core
            diff_out = run_cmd(["git", "diff", "--name-status", base_commit, "HEAD", "--", "test/core"], cwd=proposal_dir)

            changed_wast_files = set()
            for line in diff_out.splitlines():
                if not line.strip():
                    continue
                parts = line.split(maxsplit=1)
                if len(parts) < 2:
                    continue
                status, path = parts
                # We care about Added, Modified, Renamed, etc. (anything except Deleted 'D')
                if 'D' not in status:
                    if path.startswith("test/core/") and path.endswith(".wast"):
                        rel_path = path[len("test/core/"):]
                        changed_wast_files.add(rel_path)

            if not changed_wast_files:
                print(f"No added or modified tests found in test/core for proposal {name}.")
                merged_proposals.append(name)
                continue

            print(f"Found {len(changed_wast_files)} changed/added test files.")

            # 5. Build the proposal's interpreter
            interpreter_dir = os.path.join(proposal_dir, "interpreter")
            assert os.path.exists(interpreter_dir)
            print("Building proposal interpreter...")
            run_cmd(["opam", "exec", "make"], cwd=interpreter_dir)

            # 6. Convert WAST tests using proposal's build script
            out_dir = os.path.join(proposal_dir, "out")
            os.makedirs(out_dir, exist_ok=True)
            build_script = os.path.join(proposal_dir, "test", "build.py")
            assert os.path.exists(build_script)
            print("Converting WAST tests to WPT format...")
            run_cmd([build_script, "--dont-recompile", "--html", out_dir], cwd=proposal_dir)

            # 7. Create target directory in WPT proposals
            target_dir = os.path.join(wpt_root, "wasm", "proposals", name, "core")
            os.makedirs(target_dir, exist_ok=True)

            # 8. Copy only the files that were changed/added by the proposal
            copied_count = 0
            for rel_wast in changed_wast_files:
                print(f"Changed wast file: {rel_wast}")
                src_file = os.path.join(out_dir, rel_wast + ".js.html")
                dst_file = os.path.join(target_dir, rel_wast +  ".js.tentative.html")

                assert os.path.exists(src_file)
                os.makedirs(os.path.dirname(dst_file), exist_ok=True)
                shutil.copy2(src_file, dst_file)
                copied_count += 1

            if copied_count > 0:
                proposal_repo = f"https://github.com/WebAssembly/{name}"
                proposal_commit = run_cmd(["git", "rev-parse", "HEAD"], cwd=proposal_dir).strip()
                updated_proposals.append((name, proposal_repo, proposal_commit))

            print(f"Successfully copied {copied_count} test files to {target_dir}")

    finally:
        # 9. Write execution summary
        summary_path = os.path.join(wpt_root, "wasm", "proposals-summary.txt")
        if updated_proposals or merged_proposals:
            with open(summary_path, "w") as f:
                if updated_proposals:
                    f.write("The following Wasm proposals were successfully updated:\n")
                    for name, repo, commit in updated_proposals:
                        f.write(f"- {name} ({repo}/commit/{commit})\n")
                if updated_proposals and merged_proposals:
                    f.write("\n")
                if merged_proposals:
                    f.write("Note: The following proposals have 0 differences from the main spec and are likely fully merged. Please consider removing them from `wasm/proposals.json`:\n")
                    for name in merged_proposals:
                        f.write(f"- {name}\n")
            print(f"\nWritten proposals summary to {summary_path}")
        elif os.path.exists(summary_path):
            os.remove(summary_path)

        if os.path.exists(tmp_dir):
            shutil.rmtree(tmp_dir)

if __name__ == "__main__":
    main()
