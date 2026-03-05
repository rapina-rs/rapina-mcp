use std::process::Command;

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ErrorData as McpError},
    schemars, tool,
};

use crate::RapinaMcp;

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

impl RapinaMcp {
    #[tool(description = "Create a new Rapina project with the standard directory structure")]
    pub fn rapina_new(
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

    #[tool(
        description = "Add a resource (handler, model, migration) to an existing Rapina project"
    )]
    pub fn rapina_add(
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
    pub fn rapina_routes(
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

    #[tool(
        description = "Run rapina doctor to diagnose common issues in a Rapina project (missing config, auth misconfiguration, dependency problems)"
    )]
    pub fn rapina_doctor(
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
    pub fn rapina_openapi(
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
    pub fn rapina_codegen(
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
    pub fn rapina_migrate(
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
    pub fn rapina_test(
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
