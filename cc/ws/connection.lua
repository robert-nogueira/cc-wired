local M = {}

---@return ccTweaked.http.Websocket
function M.connect()
  local socket, err = http.websocket("ws://127.0.0.1:8080")

  if not socket then
    error(err)
  end

  return socket
end

function M.teste()
  print("aaa")
end

return M
