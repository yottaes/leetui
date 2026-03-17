use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::api::types::QuestionDetail;

pub fn scaffold_go(workspace: &PathBuf, detail: &QuestionDetail) -> Result<PathBuf> {
    let dir_name = format!(
        "{}-{}",
        detail.frontend_question_id,
        detail.title_slug
    );
    let project_dir = workspace.join(&dir_name);
    let solution_file = project_dir.join("solution.go");

    // Idempotent: skip if already exists
    if solution_file.exists() {
        return Ok(solution_file);
    }

    std::fs::create_dir_all(&project_dir)
        .with_context(|| format!("Failed to create dir {}", project_dir.display()))?;

    // Initialize Go module
    let output = Command::new("go")
        .args(["mod", "init", &format!("leetcode/{}", detail.title_slug)])
        .current_dir(&project_dir)
        .output()
        .context("Failed to run go mod init")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("go mod init failed: {}", stderr);
    }

    // Build the source file content
    let mut src = String::new();

    // Problem description as comments
    src.push_str(&format!("// {}: {}\n", detail.frontend_question_id, detail.title));
    src.push_str(&format!("// Difficulty: {}\n", detail.difficulty));
    src.push_str(&format!(
        "// https://leetcode.com/problems/{}/\n",
        detail.title_slug
    ));
    src.push_str("//\n");

    // Add description as comments
    if let Some(ref html) = detail.content {
        let text = html2text::from_read(html.as_bytes(), 80)
            .unwrap_or_default();
        for line in text.lines().take(50) {
            src.push_str(&format!("// {}\n", line));
        }
    }

    src.push_str("\npackage main\n\nimport \"fmt\"\n\n");

    // Code snippet
    let snippet = detail
        .code_snippets
        .as_ref()
        .and_then(|snippets| snippets.iter().find(|s| s.lang_slug == "golang"))
        .map(|s| s.code.as_str());

    match snippet {
        Some(code) => src.push_str(code),
        None => src.push_str("// No Go snippet available for this problem\n"),
    }

    src.push_str("\n\nfunc main() {\n");
    src.push_str("\tfmt.Println(\"Run your solution here\")\n");
    src.push_str("}\n");

    std::fs::write(&solution_file, src)
        .with_context(|| format!("Failed to write {}", solution_file.display()))?;

    Ok(solution_file)
}
