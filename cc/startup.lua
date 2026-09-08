local ws = require("ws")

parallel.waitForAny(function()
  shell.run("shell")
end, function()
  local socket = ws.connection.connect()
  while true do
    local r = socket.receive()
    if r ~= nil then
      print(r)
    end
  end
end)
