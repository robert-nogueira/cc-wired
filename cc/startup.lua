local ws = require("ws")

parallel.waitForAny(function()
  shell.run("shell")
end, function()
  local client = ws.connection.connect()
  while true do
    local r = client.socket.receive()
    if r ~= nil then
      print(r)
    end
  end
end)
