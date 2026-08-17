# GitHub Repo Activities

Reusable JS activities for GitHub repository writes. They implement the
`obelisk-components:github/repo` interface as three standalone activities that a
workflow can chain into a durable commit sequence:

| FFQN | Signature | Returns |
|------|-----------|---------|
| `obelisk-components:github/repo.create-branch` | `(owner, repo, base-branch, new-branch)` | head commit OID of the new branch |
| `obelisk-components:github/repo.push-file` | `(owner, repo, branch, expected-head-oid, path, content, headline, body)` | new commit OID |
| `obelisk-components:github/repo.create-pr` | `(owner, repo, head-branch, base-branch, title, body)` | PR `html_url` |

`create-branch` resolves the base branch SHA and creates `refs/heads/<new-branch>`.
`push-file` commits a single file via GitHub's `createCommitOnBranch` GraphQL
mutation (server-signed), returning the new head OID so the caller can thread it
into the next `push-file` call. `create-pr` opens (or reuses) a PR via the REST API.
Each returns `result<string, string>`.

## Prerequisites

A GitHub token with write access to the target repository (fine-grained token with
`contents` and `pull_requests` write, or a classic token with `repo` scope). The
token must be available as the `GITHUB_TOKEN` environment variable.

```sh
export GITHUB_TOKEN="..."
```

## Running the activities

These are JS activities, so there is no build step. Run Obelisk with
`obelisk-local.toml`.

```sh
obelisk server run --server-config ./server.toml --deployment ./obelisk-local.toml
```

In another terminal, drive a commit chain:

```sh
# Create a branch off main; capture the returned head OID.
obelisk execution submit -f repo.create-branch -- '"owner"' '"repo"' '"main"' '"my-branch"'

# Commit a file onto the branch, passing the previous head OID.
obelisk execution submit -f repo.push-file -- \
  '"owner"' '"repo"' '"my-branch"' '"<head-oid>"' '"README.md"' '"hello"' '"Add README"' '""'

# Open the PR.
obelisk execution submit -f repo.create-pr -- \
  '"owner"' '"repo"' '"my-branch"' '"main"' '"My PR"' '""'
```
