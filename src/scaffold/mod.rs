pub mod go;
pub mod rust;

use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::api::types::QuestionDetail;

pub fn scaffold_problem(
    workspace: &PathBuf,
    detail: &QuestionDetail,
    language: &str,
) -> Result<PathBuf> {
    match language {
        "rust" => rust::scaffold_rust(workspace, detail),
        "go" | "golang" => go::scaffold_go(workspace, detail),
        _ => bail!("Unsupported language for scaffolding: {}", language),
    }
}
