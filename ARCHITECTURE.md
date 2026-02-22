# leetcode-cli

Terminal-based LeetCode client built with Rust + Ratatui.

## Flow

```
leetcode-cli
     │
     ▼
┌──────────┐    no config found     ┌────────────┐
│  Launch   │ ────────────────────► │   Setup     │
│           │                       │  Wizard     │
└──────────┘                       └────────────┘
     │                                   │
     │  config exists                    │ saves ~/.leetcode-cli/config.toml
     ▼                                   ▼
┌──────────────────────────────────────────┐
│             Problem Browser              │
│                                          │
│  #   Title                Diff   Status  │
│  1   Two Sum              Easy   ✓       │
│  2   Add Two Numbers      Med            │
│  3   Longest Substring    Med    ✓       │
│  ...                                     │
│                                          │
│  [/] search  [d] difficulty  [t] topic   │
│  [s] status  [Enter] select  [q] quit    │
└──────────────────────────────────────────┘
     │
     │  Enter
     ▼
┌──────────────────────────────────────────┐
│            Problem Detail                │
│                                          │
│  27. Remove Element [Easy]               │
│                                          │
│  Given an integer array nums and an      │
│  integer val, remove all occurrences...  │
│                                          │
│  Example 1:                              │
│  Input: nums = [3,2,2,3], val = 3       │
│  Output: 2, nums = [2,2,_,_]            │
│                                          │
│  [o] open in editor  [b] back  [q] quit │
└──────────────────────────────────────────┘
     │
     │  'o'
     ▼
  1. Scaffolds project in workspace_dir/
  2. Opens in configured editor (nvim/code/etc)
  3. User solves the problem
  4. User returns to TUI, presses [s] to submit
```

## Architecture

```
leetcode-cli/
├── Cargo.toml
└── src/
    ├── main.rs              # entry point, app loop
    ├── app.rs               # App state machine (Setup → Browse → Detail → Solve)
    ├── config.rs            # config.toml read/write
    ├── api/
    │   ├── mod.rs
    │   ├── client.rs        # HTTP client (reqwest), session/cookies
    │   ├── queries.rs       # GraphQL query strings
    │   └── types.rs         # API response structs
    ├── ui/
    │   ├── mod.rs
    │   ├── setup.rs         # first-run wizard
    │   ├── browser.rs       # problem list table
    │   ├── detail.rs        # problem description view
    │   └── status_bar.rs    # bottom bar with keybindings
    └── scaffold/
        ├── mod.rs           # dispatcher: pick scaffolder by language
        └── rust.rs          # cargo init + inject snippet + problem comment
```

## State Machine

```
  Setup ──► Browse ◄──► Detail ──► Solve
              ▲                      │
              └──────────────────────┘
```

Each state owns its own input handling and render logic.
`App` holds the current state enum + shared data (config, problem cache).

## Config

`~/.leetcode-cli/config.toml`

```toml
workspace_dir = "~/code/leetcode"
language = "rust"
editor = "nvim"

[auth]
leetcode_session = "..."    # LEETCODE_SESSION cookie
csrf_token = "..."          # csrftoken cookie
```

Auth is cookie-based. User grabs `LEETCODE_SESSION` and `csrftoken` from browser devtools.
No OAuth/password — LeetCode doesn't have a public auth API.

## API

All via `https://leetcode.com/graphql`. Key queries:

| Action | Query | Auth? |
|---|---|---|
| List all problems | `problemsetQuestionList` | No |
| Problem detail + snippets | `question(titleSlug)` | No |
| Submit solution | `submit` mutation | Yes |
| Check submission result | `submissionDetails` | Yes |

Fetching problems doesn't need auth. Submitting does.

## Scaffold: Rust Example

When user picks problem 27 (Remove Element) with language=rust:

```
~/code/leetcode/27-remove-element/
├── Cargo.toml
└── src/
    └── main.rs
```

`main.rs` generated content:

```rust
// 27. Remove Element [Easy]
// https://leetcode.com/problems/remove-element/
//
// Given an integer array nums and an integer val,
// remove all occurrences of val in nums in-place.
// ...

fn main() {
    println!("27. Remove Element");
}

struct Solution;

impl Solution {
    pub fn remove_element(nums: &mut Vec<i32>, val: i32) -> i32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        let mut nums = vec![3, 2, 2, 3];
        let k = Solution::remove_element(&mut nums, 3);
        assert_eq!(k, 2);
    }
}
```

## Dependencies

```toml
[dependencies]
ratatui = "0.29"          # TUI framework
crossterm = "0.28"        # terminal backend
reqwest = { version = "0.12", features = ["cookies", "json"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"              # config parsing
dirs = "6"                # ~/.leetcode-cli path
html2text = "0.14"        # convert problem HTML → terminal-friendly text
```

## MVP Scope

Phase 1 — read-only, no auth:
- [ ] Config wizard (workspace dir, language, editor)
- [ ] Fetch + cache problem list
- [ ] Problem browser with search, difficulty filter
- [ ] Problem detail view (HTML → rendered text)
- [ ] Scaffold Rust project with snippet + description
- [ ] Open in editor

Phase 2 — auth + submit:
- [ ] Cookie-based auth setup
- [ ] Submit solution from TUI
- [ ] Show submission result (accepted/wrong answer/TLE + runtime/memory)

Phase 3 — multi-language + extras:
- [ ] Go, JavaScript scaffolders
- [ ] Track solved/unsolved status locally
- [ ] Daily challenge highlight
- [ ] Problem tags/topics filter
