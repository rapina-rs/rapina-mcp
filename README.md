# rapina-mcp

MCP (Model Context Protocol) server for [Rapina](https://github.com/rapina-rs/rapina) framework.

Gives AI assistants native understanding of Rapina projects — scaffold, inspect, diagnose, and introspect your application without leaving the conversation.

## Install

```bash
cargo install rapina-mcp
```

## Setup

Add to your MCP client configuration:

**Claude Code** (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "rapina": {
      "command": "rapina-mcp"
    }
  }
}
```

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "rapina": {
      "command": "rapina-mcp"
    }
  }
}
```

Works with any MCP-compatible client (Cursor, Windsurf, etc.) using the same pattern.

## Tools

| Tool | Description |
|------|-------------|
| `rapina_new` | Scaffold a new Rapina project |
| `rapina_add` | Add a resource (handler, model, migration) |
| `rapina_routes` | List all defined routes |
| `rapina_doctor` | Run project diagnostics |
| `rapina_openapi` | Generate OpenAPI specification |
| `rapina_codegen` | Run code generation |
| `rapina_migrate` | Run, rollback, or check migration status |
| `rapina_test` | Run project tests |
| `rapina_explain` | Introspect project structure and return a full architectural summary |

Most tools wrap the `rapina` CLI, so you need it installed (`cargo install rapina-cli`). The `rapina_explain` tool works standalone by reading the filesystem directly.

## Testing

Verify the server works with the MCP inspector:

```bash
npx @modelcontextprotocol/inspector rapina-mcp
```

## License

MIT
