# TODO - The TODO list for this ltlsp.

The current plan is to use spelling and grammar checking from LanguageTool for comments supplement
this with regexes to assist with edge cases. Use ZSpell to spell check strings.

## Main tooling
- [x] Setup tracing to log to a file
- [ ] Make tracing file not specific to unique setup
- [ ] Make tracing file configurable
- [x] Create container file with setup to allow building of grammars and tooling

## Tooling
- [x] Setup Dev Containers
- [x] Setup LanguageTool container
- [x] Setup github build to push container to ghcr.io
	- [x] Make repository public
	- [x] Make build process to create container
	- [x] Push container to ghcr.io
- [x] Setup LanguageTool container
- [ ] Setup container to run whole ltlsp solution

## Language Server
- [x] Run a language server
- [x] Successfully communicate with Helix editor
- [x] Send file content for parsing (whole file)
- [ ] Send incremental file content for parsing (diff updates)
- [x] Send contents to Treesitter for parsing
- [ ] Update tree sitter with differential update
- [x] Send parsed contents to LanguageTool for checking
- [x] Send LSP client LanguageTool suggestions
- [x] Implement suggestions from LSP client **Potentially large amount of work**

## TreeSitter
- [x] Install rust grammar.
- [x] Read rust file using tree-sitter grammar.
- [x] Read rust file and get line comments.
- [x] Setup comments configuration from a config file.
- [x] Setup config to setup grammars for use.
- [x] Build grammars to support multiple languages.
- [x] Setup build.rs to build grammars for multiple languages. 
- [ ] Support nested grammars (needed for markdown).

## LanguageTool
- [x] Add Requests that will LanguageTool to check spelling
- [x] Run LanguageTool locally and then shutdown on Dispose of Runner
- [x] Add regular expression to remove old comment markers interfering with grammar.
- [ ] Refactor - Separate LanguageTool remote server and local server support.
- [ ] Add support for LanguageTool professional version.
- [x] Add Podman container support that will let the LanguageTool be started and stopped and easily.
- [ ] Add Docker container support that will let the LanguageTool be started and stopped and easily.
- [ ] Add Incus container support that will let the LanguageTool be started and stopped and easily.

## ZSpell
- [ ] Add ZSpell support.
- [ ] Download and install dictionaries for ZSpell.
- [ ] Spell check languages.
- [ ] Construct updates for ZSpell.

## Other spell checkers
- [ ] Include support for ZSpell or Hunspell etc.
