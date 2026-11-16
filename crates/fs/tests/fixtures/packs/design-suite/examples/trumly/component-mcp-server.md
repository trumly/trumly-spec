---
id: component-mcp-server
type: component
title: MCP server
plane: sd--design
parent: group-tools
fields:
  kind: service
  owner: trumly
  status: built
links:
- type: depends-on
  to: component-core
- type: depends-on
  to: component-fs
- type: exposes
  to: interface-mcp-tools
- type: reads
  to: datastore-model-folder
- type: runs-in
  to: environment-dev-machine
- type: satisfies
  to: requirement-agent-access
- type: satisfies
  to: requirement-agent-overview
---
