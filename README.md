# LeetCode TUI

A terminal-based interface for browsing, solving, and submitting LeetCode problems, built with **Rust** and **Ratatui**.

> **Note:** This is a hobby project. I am currently focusing strictly on **Rust** support.

![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)

## Features

- **Browse Problems**: List view with filtering (Difficulty, Status) and search.
- **Read**: Render problem descriptions (HTML to TUI) directly in the terminal.
- **Scaffold**: Automatically generate a Rust project with boilerplate for the selected problem.
- **Test & Submit**: Run code against LeetCode's test cases and submit solutions without leaving the CLI.
- **Manage Lists**: Create and manage custom favorite lists.
- **Stats**: View user profile statistics.

## Setup & Authentication

This tool works by parsing your existing LeetCode session cookies directly from your web browser.

1.  **Build the Project**

    ```bash
    cargo build --release
    ```

2.  **Run with Permissions**
    Because the application needs to read protected browser storage (cookies, check browser_login() function in app.rs) to log you in, you must run it with sufficient privileges.
    - **Option A (Recommended for ease):** Run with `sudo`.
      ```bash
      sudo ./target/release/leetcode-tui
      ```
    - **Option B (Manual):** Run without sudo. Your operating system (macOS/Linux) may repeatedly prompt you to access the system Keyring or browser directories for every browser installed on your system.

## Controls

| Key       | Action                        |
| :-------- | :---------------------------- |
| `j` / `k` | Navigate lists                |
| `Enter`   | View details / Select         |
| `/`       | Search problems               |
| `f`       | Filter (Difficulty, Solved)   |
| `o`       | **Scaffold** & open in editor |
| `r`       | **Run** code                  |
| `s`       | **Submit** solution           |
| `L`       | Manage Lists                  |
| `q`       | Quit                          |

## Contributing

Contributions are welcome! Since this is a hobby project, I am prioritizing **Rust-specific features**.

- Feel free to **fork** the repo and modify it as you wish.
- If you have a fix or feature, submit a **PR** directly.

## License

This project is open source and available under the [MIT License](LICENSE).
