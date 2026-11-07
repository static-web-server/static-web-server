# Posting a Pending Review with `gh`

Companion to `SKILL.md` step 9. GitHub allows one pending review per user per PR.

## Find the user's pending review

```bash
gh api graphql -f query='{repository(owner:"<o>",name:"<r>"){pullRequest(number:<N>){id reviews(states:PENDING,first:1){nodes{id}}}}}'
```

## No pending review: create one

Omit `event` so the review stays `PENDING`. `review.json`:

```json
{
  "commit_id": "<head sha>",
  "body": "<general notes, or omit>",
  "comments": [
    { "path": "src/handler.rs", "line": 120, "side": "RIGHT", "body": "..." }
  ]
}
```

```bash
gh api repos/<o>/<r>/pulls/<N>/reviews -X POST --input review.json
```

A pending review created with an empty `body` **cannot** get one later (422 "missing body"). Put general notes (rebase, conflicts, hygiene) in a file-level thread instead, or leave them for the maintainer's submit text.

## Pending review exists: append a thread

```bash
gh api graphql -f query='mutation($rid:ID!,$path:String!,$line:Int!,$body:String!){
  addPullRequestReviewThread(input:{pullRequestReviewId:$rid,path:$path,line:$line,side:RIGHT,body:$body}){thread{id}}}' \
  -f rid=<PRR_id> -f path=<file> -F line=<n> -f body="$(cat comment.md)"
```

- `line` is the **new-file** line number at the PR head commit
- Multi-line: add `startLine:<n>` (and `startSide:RIGHT`)
- File-level: `subjectType:FILE` and no `line`
- Write each body to a file and pass `-f body="$(cat f)"`; inline strings break on backticks and code blocks

## Verify

```bash
gh api graphql -f query='{repository(owner:"<o>",name:"<r>"){pullRequest(number:<N>){reviews(states:PENDING,first:1){nodes{comments(first:50){nodes{path line body}}}}}}}'
```

Every comment must sit on the intended `path:line`; a comment on a line outside the diff is silently attached to the nearest hunk.
