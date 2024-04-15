# TODO - The TODO list for this ltlsp.

## Main tooling
- [x] Setup tracing to log to a file
- [ ] Make tracing file not specific to unique setup
- [ ] Make tracing file configurable
- [ ] Create container file with setup to allow building of grammars and tooling

## Tooling
- [ ] Setup Dev Containers
- [x] Setup LanguageTool container
- [ ] Setup github build to push container to ghcr.io
	- [ ] Make repository public
	- [x] Make build process to create container
	- [ ] Push container to ghcr.io
- [x] Setup LanguageTool container
- [ ] Setup container to run whole ltlsp solution

## Language Server
- [x] Run a language server
- [x] Successfully communicate with Helix editor
- [x] Send file content for parsing (whole file)
- [ ] Send incremental file content for parsing (diff updates)
- [ ] Send contents to Treesitter for parsing
- [ ] Update tree sitter with differential update
- [ ] Send parsed contents to LanguageTool for checking
- [ ] Send LSP client LanguageTool suggestions
- [ ] Implement suggestions from LSP client **Potentially large amount of work**

## TreeSitter
- [x] Install rust grammar
- [x] Read rust file using tree-sitter grammar
- [x] Read rust file and get line comments
- [ ] Setup comments configuration from a config file
- [ ] Setup config to setup grammars for use
- [ ] Build grammars to support multiple languages
- [ ] Support nested grammars (needed for markdown)

## LanguageTool
- [x] Add Requests that will LanguageTool to check spelling
- [ ] Add support for LanguageTool professional version
- [ ] Add Podman container support that will let the LanguageTool be started and stopped and easily
- [ ] Add Docker container support that will let the LanguageTool be started and stopped and easily
- [ ] Add Incus container support that will let the LanguageTool be started and stopped and easily

## Other spell checkers
- [ ] Include support for ZSpell or Hunspell etc.
