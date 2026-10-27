defmodule StatusServer.Router do
  use Plug.Router

  alias StatusServer.ServerManager
  alias StatusServer.Minecraft.Ping

  plug :cors
  plug :match
  plug Plug.Parsers, parsers: [:json], pass: ["application/json"], json_decoder: Jason
  plug :dispatch

  get "/ws" do
    WebSockAdapter.upgrade(conn, StatusServer.SocketHandler, [], [])
  end

  get "/health" do
    send_json(conn, 200, %{status: "ok"})
  end

  get "/api/servers" do
    servers = ServerManager.list_servers()
    send_json(conn, 200, %{servers: servers})
  end

  post "/api/servers" do
    case ServerManager.add_server(conn.body_params) do
      {:ok, server} ->
        send_json(conn, 201, %{server: server})

      {:error, reason} ->
        send_json(conn, 400, %{error: to_string(reason)})
    end
  end

  delete "/api/servers/:id" do
    ServerManager.remove_server(id)
    send_json(conn, 200, %{deleted: id})
  end

  get "/api/ping" do
    conn = fetch_query_params(conn)
    raw_host = Map.get(conn.params, "host", "localhost")

    {extracted_host, default_port} =
      if String.contains?(raw_host, ":") do
        case String.split(raw_host, ":", parts: 2) do
          [h, p] ->
            case Integer.parse(p) do
              {parsed_p, _} -> {h, parsed_p}
              :error -> {h, 25565}
            end
          _ ->
            {raw_host, 25565}
        end
      else
        {raw_host, 25565}
      end

    port =
      case Map.get(conn.params, "port") do
        nil ->
          default_port
        p when is_integer(p) ->
          p
        p when is_binary(p) ->
          case Integer.parse(p) do
            {parsed_p, _} -> parsed_p
            :error -> default_port
          end
      end

    case Ping.ping(extracted_host, port) do
      {:ok, status} ->
        send_json(conn, 200, %{host: extracted_host, port: port, status: status})

      {:error, reason} ->
        send_json(conn, 200, %{host: extracted_host, port: port, status: %{online: false, reason: to_string(reason)}})
    end
  end

  match _ do
    send_json(conn, 404, %{error: "not_found"})
  end

  defp cors(conn, _opts) do
    conn
    |> put_resp_header("access-control-allow-origin", "*")
    |> put_resp_header("access-control-allow-methods", "GET, POST, DELETE, OPTIONS")
    |> put_resp_header("access-control-allow-headers", "content-type")
  end

  defp send_json(conn, status, data) do
    body = Jason.encode!(data)

    conn
    |> put_resp_header("content-type", "application/json")
    |> send_resp(status, body)
  end
end
