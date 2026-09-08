local M = {}

---@class Client
---@field socket ccTweaked.http.Websocket
---@field connected boolean

---@return Client
function M.connect()
  local ws, err = http.websocket("ws://127.0.0.1:8080")

  if not ws then
    error(err)
  end

  ---@type Client
  local client = {
    socket = ws,
    connected = true,
  }

  setmetatable(client, {
    __tostring = function(self)
      return string.format("Client { connected = %s }", tostring(self.connected))
    end,
  })

  return client
end

function M.teste()
  print("aaa")
end

return M
