defmodule StatusServer.SocketHandler do
  @behaviour WebSock

  alias StatusServer.ServerManager

  @impl true
  def init(_args) do
    Registry.register(StatusServer.PubSub, "servers", [])

    servers = ServerManager.list_servers()
    initial_payload = Jason.encode!(%{type: "initial_state", data: servers})

    {:ok, %{}, {:push, {:text, initial_payload}}}
  end

  @impl true
  def handle_in({msg, [opcode: :text]}, state) do
    case Jason.decode(msg) do
      {:ok, %{"action" => "ping_now"}} ->
        ServerManager.force_ping()
        {:ok, state}

      _ ->
        {:ok, state}
    end
  end

  @impl true
  def handle_info({:ws_broadcast, payload}, state) do
    {:push, {:text, payload}, state}
  end

  @impl true
  def terminate(_reason, _state) do
    :ok
  end
end
