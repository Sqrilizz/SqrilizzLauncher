defmodule StatusServer.ServerManager do
  use GenServer

  alias StatusServer.Minecraft.Ping

  @ping_interval 5000

  def start_link(init_arg) do
    GenServer.start_link(__MODULE__, init_arg, name: __MODULE__)
  end

  def list_servers do
    GenServer.call(__MODULE__, :list_servers)
  end

  def add_server(params) do
    GenServer.call(__MODULE__, {:add_server, params})
  end

  def remove_server(id) do
    GenServer.call(__MODULE__, {:remove_server, id})
  end

  def force_ping do
    send(__MODULE__, :ping_all)
  end

  @impl true
  def init(_opts) do
    initial_servers = [
      %{
        id: "hypixel",
        name: "Hypixel Network",
        host: "mc.hypixel.net",
        port: 25565,
        status: %{online: false, latency_ms: nil}
      }
    ]

    schedule_ping(100)

    {:ok, %{servers: initial_servers}}
  end

  @impl true
  def handle_call(:list_servers, _from, state) do
    {:reply, state.servers, state}
  end

  @impl true
  def handle_call({:add_server, params}, _from, state) do
    id = Map.get(params, "id") || Map.get(params, :id) || generate_id(params)
    name = Map.get(params, "name") || Map.get(params, :name) || "Minecraft Server"
    host = Map.get(params, "host") || Map.get(params, :host) || "localhost"
    port = Map.get(params, "port") || Map.get(params, :port) || 25565

    port = if is_binary(port), do: String.to_integer(port), else: port

    new_server = %{
      id: to_string(id),
      name: to_string(name),
      host: to_string(host),
      port: port,
      status: %{online: false, latency_ms: nil}
    }

    updated_servers = [new_server | Enum.reject(state.servers, &(&1.id == new_server.id))]

    schedule_ping(50)

    {:reply, {:ok, new_server}, %{state | servers: updated_servers}}
  end

  @impl true
  def handle_call({:remove_server, id}, _from, state) do
    target_id = to_string(id)
    updated_servers = Enum.reject(state.servers, &(&1.id == target_id))
    {:reply, :ok, %{state | servers: updated_servers}}
  end

  @impl true
  def handle_info(:ping_all, state) do
    updated_servers =
      state.servers
      |> Task.async_stream(
        fn server ->
          status =
            case Ping.ping(server.host, server.port) do
              {:ok, info} ->
                info

              {:error, _reason} ->
                %{online: false, latency_ms: nil}
            end

          Map.put(server, :status, status)
        end,
        max_concurrency: 16,
        timeout: 6000
      )
      |> Enum.map(fn
        {:ok, server} -> server
        _ -> nil
      end)
      |> Enum.reject(&is_nil/1)

    broadcast_update(updated_servers)
    schedule_ping(@ping_interval)

    {:noreply, %{state | servers: updated_servers}}
  end

  defp schedule_ping(delay) do
    Process.send_after(self(), :ping_all, delay)
  end

  defp generate_id(params) do
    host = Map.get(params, "host") || Map.get(params, :host) || "srv"
    port = Map.get(params, "port") || Map.get(params, :port) || 25565
    "#{host}_#{port}"
  end

  defp broadcast_update(servers) do
    payload = Jason.encode!(%{type: "servers_update", data: servers})

    Registry.dispatch(StatusServer.PubSub, "servers", fn entries ->
      for {pid, _} <- entries do
        send(pid, {:ws_broadcast, payload})
      end
    end)
  end
end
