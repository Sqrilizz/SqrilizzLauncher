defmodule StatusServer do
  alias StatusServer.Minecraft.Ping
  alias StatusServer.ServerManager

  def ping(host, port \\ 25565, timeout \\ 4000) do
    Ping.ping(host, port, timeout)
  end

  def list_servers do
    ServerManager.list_servers()
  end

  def add_server(params) do
    ServerManager.add_server(params)
  end

  def remove_server(id) do
    ServerManager.remove_server(id)
  end
end
