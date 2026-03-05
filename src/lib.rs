use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ErrorData as McpError, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NewProjectParams {
    #[schemars(description = "Name of the new Rapina project")]
    pub name: String,
    #[schemars(description = "Directory where the project will be created (defaults to current directory)")]
    pub path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddResourceParams {
    #[schemars(description = "Type of resource to add (e.g. handler, model, migration)")]
    pub resource_type: String,
    #[schemars(description = "Name of the resource")]
    pub name: String,
    #[schemars(description = "Path to the Rapina project root")]
    pub project_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProjectPathParams {
    #[schemars(description = "Path to the Rapina project root")]
    pub project_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MigrateParams {
    #[schemars(description = "Migration subcommand (e.g. run, rollback, status)")]
    pub action: String,
    #[schemars(description = "Path to the Rapina project root")]
    pub project_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplainParams {
    #[schemars(description = "Path to the Rapina project root")]
    pub project_path: String,
}

#[derive(Debug, Clone)]
pub struct RapinaMcp {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RapinaMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Create a new Rapina project with the standard directory structure")]
    fn rapina_new(
        &self,
        Parameters(params): Parameters<NewProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("new").arg(&params.name);
        if let Some(ref path) = params.path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Add a resource (handler, model, migration) to an existing Rapina project")]
    fn rapina_add(
        &self,
        Parameters(params): Parameters<AddResourceParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("add").arg(&params.resource_type).arg(&params.name);
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "List all routes defined in a Rapina project")]
    fn rapina_routes(
        &self,
        Parameters(params): Parameters<ProjectPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("routes");
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Run rapina doctor to diagnose common issues in a Rapina project (missing config, auth misconfiguration, dependency problems)")]
    fn rapina_doctor(
        &self,
        Parameters(params): Parameters<ProjectPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("doctor");
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Generate the OpenAPI specification for a Rapina project")]
    fn rapina_openapi(
        &self,
        Parameters(params): Parameters<ProjectPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("openapi");
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Run code generation for a Rapina project")]
    fn rapina_codegen(
        &self,
        Parameters(params): Parameters<ProjectPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("codegen");
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Run database migrations for a Rapina project")]
    fn rapina_migrate(
        &self,
        Parameters(params): Parameters<MigrateParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("migrate").arg(&params.action);
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Run tests in a Rapina project")]
    fn rapina_test(
        &self,
        Parameters(params): Parameters<ProjectPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut cmd = Command::new("rapina");
        cmd.arg("test");
        if let Some(ref path) = params.project_path {
            cmd.current_dir(path);
        }
        run_command(cmd)
    }

    #[tool(description = "Introspect a Rapina project and return a structured summary of its architecture: modules, routes, middleware, auth configuration, database setup, and dependencies")]
    fn rapina_explain(
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

        let src_dir = root.join("src");
        if src_dir.exists() {
            report.push_str("## Project Structure\n\n");
            report.push_str(&walk_source_tree(&src_dir, 0));
            report.push('\n');
        }

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

        let migrations_dir = root.join("migrations");
        if migrations_dir.exists() {
            report.push_str("## Migrations\n\n");
            if let Ok(entries) = fs::read_dir(&migrations_dir) {
                let mut files: Vec<String> = entries
                    .flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect();
                files.sort();
                for f in &files {
                    report.push_str(&format!("- {}\n", f));
                }
            }
            report.push('\n');
        }

        report.push_str("## Configuration Files\n\n");
        for name in ["rapina.toml", "Rapina.toml", ".env", ".env.example", "config.toml"] {
            if root.join(name).exists() {
                report.push_str(&format!("- {}\n", name));
            }
        }
        report.push('\n');

        Ok(CallToolResult::success(vec![Content::text(report)]))
    }
}

#[tool_handler]
impl ServerHandler for RapinaMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::from_build_env())
            .with_instructions(
                "MCP server for the Rapina web framework. \
                 Provides tools to scaffold projects, inspect routes, \
                 run diagnostics, generate code, and introspect Rapina applications."
                    .to_string(),
            )
    }
}

fn run_command(mut cmd: Command) -> Result<CallToolResult, McpError> {
    let output = cmd.output().map_err(|e| {
        McpError::internal_error(format!("Failed to execute rapina CLI: {e}"), None)
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let mut text = stdout.into_owned();
        if !stderr.is_empty() {
            text.push_str("\n--- stderr ---\n");
            text.push_str(&stderr);
        }
        Ok(CallToolResult::success(vec![Content::text(text)]))
    } else {
        let mut text = String::new();
        if !stdout.is_empty() {
            text.push_str(&stdout);
            text.push('\n');
        }
        text.push_str(&stderr);
        Ok(CallToolResult::error(vec![Content::text(text)]))
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
            let rapina_deps: Vec<&str> = deps
                .keys()
                .filter(|k| k.starts_with("rapina"))
                .map(|s| s.as_str())
                .collect();
            if !rapina_deps.is_empty() {
                summary.push_str(&format!("- **rapina deps:** {}\n", rapina_deps.join(", ")));
            }
            summary.push_str(&format!("- **total dependencies:** {}\n", deps.len()));
        }
    }
    summary
}

fn walk_source_tree(dir: &Path, depth: usize) -> String {
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
            output.push_str(&walk_source_tree(&path, depth + 1));
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
            if !matches!(name.as_str(), "middleware" | "config" | "common" | "utils") {
                modules.push(name);
            }
        }
    }
    modules.sort();
    modules
}
