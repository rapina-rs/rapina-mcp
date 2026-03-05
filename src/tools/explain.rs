use std::fs;
use std::path::{Path, PathBuf};

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData as McpError},
    schemars, tool,
};

use crate::RapinaMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainParams {
    #[schemars(description = "Path to the Rapina project root")]
    pub project_path: String,
}

impl RapinaMcp {
    #[tool(
        description = "Introspect a Rapina project and return a structured summary of its architecture: modules, routes, middleware, auth configuration, database setup, and dependencies. This is the AI-native tool for understanding a Rapina codebase at a glance."
    )]
    pub fn rapina_explain(
        &self,
        Parameters(params): Parameters<ExplainParams>,
    ) -> Result<CallToolResult, McpError> {
        let root = PathBuf::from(&params.project_path);

        if !root.exists() {
            return Err(McpError::invalid_params(
                format!("Project path does not exist: {}", params.project_path),
                None,
            ));
        }

        let mut report = String::new();
        report.push_str(&format!("# Rapina Project: {}\n\n", params.project_path));

        // Cargo.toml
        let cargo_path = root.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_path) {
                report.push_str("## Cargo.toml\n\n");
                report.push_str(&extract_cargo_summary(&content));
                report.push('\n');
            }
        } else {
            report.push_str("**Warning:** No Cargo.toml found. Is this a Rapina project?\n\n");
        }

        // Source structure
        let src_dir = root.join("src");
        if src_dir.exists() {
            report.push_str("## Project Structure\n\n");
            report.push_str(&walk_source_tree(&src_dir, &src_dir, 0));
            report.push('\n');
        }

        // Module files — look for handlers, routes, models
        report.push_str("## Modules Detected\n\n");
        let modules = detect_modules(&src_dir);
        if modules.is_empty() {
            report.push_str("No feature modules detected.\n\n");
        } else {
            for module in &modules {
                report.push_str(&format!("- **{}**\n", module));
            }
            report.push('\n');
        }

        // Middleware
        let middleware_dir = src_dir.join("middleware");
        if middleware_dir.exists() {
            report.push_str("## Middleware\n\n");
            if let Ok(entries) = fs::read_dir(&middleware_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.ends_with(".rs") && name != "mod.rs" {
                        report.push_str(&format!("- {}\n", name.trim_end_matches(".rs")));
                    }
                }
            }
            report.push('\n');
        }

        // Migrations
        let migrations_dir = root.join("migrations");
        if migrations_dir.exists() {
            report.push_str("## Migrations\n\n");
            if let Ok(entries) = fs::read_dir(&migrations_dir) {
                let mut migration_files: Vec<String> = entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect();
                migration_files.sort();
                for f in &migration_files {
                    report.push_str(&format!("- {}\n", f));
                }
            }
            report.push('\n');
        }

        // Config files
        report.push_str("## Configuration Files\n\n");
        let config_files = [
            "rapina.toml",
            "Rapina.toml",
            ".env",
            ".env.example",
            "config.toml",
        ];
        for name in &config_files {
            if root.join(name).exists() {
                report.push_str(&format!("- {}\n", name));
            }
        }
        report.push('\n');

        Ok(CallToolResult::success(vec![Content::text(report)]))
    }
}

fn extract_cargo_summary(content: &str) -> String {
    let mut summary = String::new();

    if let Ok(doc) = content.parse::<toml::Table>() {
        if let Some(package) = doc.get("package").and_then(|v| v.as_table()) {
            if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                summary.push_str(&format!("- **name:** {}\n", name));
            }
            if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                summary.push_str(&format!("- **version:** {}\n", version));
            }
            if let Some(edition) = package.get("edition").and_then(|v| v.as_str()) {
                summary.push_str(&format!("- **edition:** {}\n", edition));
            }
        }

        if let Some(deps) = doc.get("dependencies").and_then(|v| v.as_table()) {
            let rapina_deps: Vec<&String> =
                deps.keys().filter(|k| k.starts_with("rapina")).collect();
            if !rapina_deps.is_empty() {
                summary.push_str("- **rapina deps:** ");
                summary.push_str(
                    &rapina_deps
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                summary.push('\n');
            }

            summary.push_str(&format!("- **total dependencies:** {}\n", deps.len()));
        }
    }

    summary
}

fn walk_source_tree(dir: &Path, base: &Path, depth: usize) -> String {
    let mut output = String::new();
    let indent = "  ".repeat(depth);

    let Ok(entries) = fs::read_dir(dir) else {
        return output;
    };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();

        if path.is_dir() {
            output.push_str(&format!("{}- **{}/**\n", indent, name));
            output.push_str(&walk_source_tree(&path, base, depth + 1));
        } else if name.ends_with(".rs") {
            output.push_str(&format!("{}- {}\n", indent, name));
        }
    }

    output
}

fn detect_modules(src_dir: &Path) -> Vec<String> {
    let mut modules = Vec::new();

    let Ok(entries) = fs::read_dir(src_dir) else {
        return modules;
    };

    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            // Skip common non-feature dirs
            if !matches!(name.as_str(), "middleware" | "config" | "common" | "utils") {
                modules.push(name);
            }
        }
    }

    modules.sort();
    modules
}
