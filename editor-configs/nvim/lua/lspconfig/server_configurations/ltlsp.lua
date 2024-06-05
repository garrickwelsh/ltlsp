local util = require("lspconfig.util")
local async = require("lspconfig.async")

local function is_library(fname)
  local user_home = util.path.sanitize(vim.env.HOME)
  local cargo_home = os.getenv("CARGO_HOME") or util.path.join(user_home, ".cargo")
  local registry = util.path.join(cargo_home, "registry", "src")
  local git_registry = util.path.join(cargo_home, "git", "checkouts")

  local rustup_home = os.getenv("RUSTUP_HOME") or util.path.join(user_home, ".rustup")
  local toolchains = util.path.join(rustup_home, "toolchains")

  for _, item in ipairs({ toolchains, registry, git_registry }) do
    if util.path.is_descendant(item, fname) then
      local clients = util.get_lsp_clients({ name = "rust_analyzer" })
      return #clients > 0 and clients[#clients].config.root_dir or nil
    end
  end
end

return {
  default_config = {
    cmd = { "/home/gaz/devel/ltlsp/target/debug/ltlsp" },
    filetypes = { "rust" },
    single_file_support = true,
    root_dir = function(fname)
      local reuse_active = is_library(fname)
      if reuse_active then
        return reuse_active
      end

      local cargo_crate_dir = util.root_pattern("Cargo.toml")(fname)
      local cargo_workspace_root

      if cargo_crate_dir ~= nil then
        local cmd = {
          "cargo",
          "metadata",
          "--no-deps",
          "--format-version",
          "1",
          "--manifest-path",
          util.path.join(cargo_crate_dir, "Cargo.toml"),
        }

        local result = async.run_command(cmd)

        if result and result[1] then
          result = vim.json.decode(table.concat(result, ""))
          if result["workspace_root"] then
            cargo_workspace_root = util.path.sanitize(result["workspace_root"])
          end
        end
      end

      return cargo_workspace_root
        or cargo_crate_dir
        or util.root_pattern("rust-project.json")(fname)
        or util.find_git_ancestor(fname)
    end,
    capabilities = {
      experimental = {
        serverStatusNotification = true,
      },
    },
  },
  docs = {
    description = [[
https://github.com/garrickwelsh/ltlsp

ltlsp spelling and grammar for code comments and git commits.


The settings can be used like this:
```lua
require'lspconfig'.ltlsp.setup{
  settings = {
    ['ltlsp'] = {
      diagnostics = {
        enable = false;
      }
    }
  }
}
```
    ]],
    default_config = {
      root_dir = [[root_pattern("Cargo.toml", "rust-project.json")]],
    },
  },
}
