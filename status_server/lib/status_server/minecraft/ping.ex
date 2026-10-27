defmodule StatusServer.Minecraft.Ping do
  import Bitwise

  def ping(host, port, timeout \\ 3500) do
    {target_host, target_port} = resolve_srv(host, port)
    char_target = to_charlist(target_host)
    opts = [:binary, active: false]

    t_connect_start = System.monotonic_time(:millisecond)

    case :gen_tcp.connect(char_target, target_port, opts, timeout) do
      {:ok, socket} ->
        tcp_rtt = max(System.monotonic_time(:millisecond) - t_connect_start, 1)

        try do
          handshake = handshake_packet(host, target_port)
          request = request_packet()

          with :ok <- :gen_tcp.send(socket, handshake <> request),
               {:ok, _packet_len} <- read_varint(socket, timeout),
               {:ok, 0} <- read_varint(socket, timeout),
               {:ok, str_len} <- read_varint(socket, timeout),
               {:ok, json_data} <- recv_exact(socket, str_len, timeout) do
            ping_payload = <<1::64-big>>
            ping_packet = <<0x09, 0x01>> <> ping_payload
            t_ping_start = System.monotonic_time(:millisecond)

            latency =
              case :gen_tcp.send(socket, ping_packet) do
                :ok ->
                  case read_varint(socket, 1500) do
                    {:ok, _pong_len} ->
                      case read_varint(socket, 1500) do
                        {:ok, 1} ->
                          case recv_exact(socket, 8, 1500) do
                            {:ok, _} ->
                              max(System.monotonic_time(:millisecond) - t_ping_start, 1)

                            _ ->
                              tcp_rtt
                          end

                        _ ->
                          tcp_rtt
                      end

                    _ ->
                      tcp_rtt
                  end

                _ ->
                  tcp_rtt
              end

            case Jason.decode(json_data) do
              {:ok, parsed} ->
                {:ok, format_result(parsed, latency)}

              {:error, _} ->
                {:ok, %{online: true, latency_ms: latency, raw: json_data}}
            end
          else
            _ ->
              {:error, :protocol_error}
          end
        after
          :gen_tcp.close(socket)
        end

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp resolve_srv(host, port) do
    if port == 25565 do
      srv_query = ~c"_minecraft._tcp." ++ to_charlist(host)

      case :inet_res.getbyname(srv_query, :srv) do
        {:ok, {:hostent, _, _, :srv, _, [{_prio, _weight, srv_port, srv_target} | _]}} ->
          {to_string(srv_target), srv_port}

        _ ->
          {host, port}
      end
    else
      {host, port}
    end
  end

  defp format_result(data, latency) do
    players = Map.get(data, "players", %{})
    version = Map.get(data, "version", %{})
    desc = Map.get(data, "description", "")
    favicon = Map.get(data, "favicon")

    motd =
      desc
      |> clean_motd()
      |> String.trim()

    %{
      online: true,
      latency_ms: latency,
      motd: motd,
      favicon: favicon,
      players: %{
        online: Map.get(players, "online", 0),
        max: Map.get(players, "max", 0),
        sample: Map.get(players, "sample", [])
      },
      version: %{
        name: Map.get(version, "name", "Unknown"),
        protocol: Map.get(version, "protocol", 0)
      }
    }
  end

  defp clean_motd(%{"text" => text, "extra" => extra}) when is_list(extra) do
    extra_text = Enum.map_join(extra, "", &clean_motd/1)
    to_string(text) <> extra_text
  end

  defp clean_motd(%{"text" => text}), do: to_string(text)
  defp clean_motd(text) when is_binary(text), do: text
  defp clean_motd(_), do: ""

  defp strip_color_codes(str) do
    str
  end

  defp handshake_packet(host, port) do
    host_bytes = :unicode.characters_to_binary(host)

    payload =
      <<0x00::8>> <>
        encode_varint(47) <>
        encode_varint(byte_size(host_bytes)) <>
        host_bytes <>
        <<port::16-big>> <>
        encode_varint(1)

    encode_varint(byte_size(payload)) <> payload
  end

  defp request_packet do
    payload = <<0x00::8>>
    encode_varint(byte_size(payload)) <> payload
  end

  def encode_varint(val) when val < 0 do
    encode_varint(val + 4_294_967_296)
  end

  def encode_varint(val) when val < 128 do
    <<val::8>>
  end

  def encode_varint(val) do
    <<1::1, band(val, 0x7F)::7, encode_varint(bsr(val, 7))::binary>>
  end

  def read_varint(socket, timeout, acc \\ 0, shift \\ 0) do
    case :gen_tcp.recv(socket, 1, timeout) do
      {:ok, <<0::1, val::7>>} ->
        {:ok, bor(acc, bsl(val, shift))}

      {:ok, <<1::1, val::7>>} ->
        read_varint(socket, timeout, bor(acc, bsl(val, shift)), shift + 7)

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp recv_exact(socket, length, timeout) do
    recv_loop(socket, length, <<>>, timeout)
  end

  defp recv_loop(_socket, 0, acc, _timeout), do: {:ok, acc}

  defp recv_loop(socket, remaining, acc, timeout) do
    case :gen_tcp.recv(socket, remaining, timeout) do
      {:ok, data} ->
        new_acc = acc <> data
        recv_loop(socket, remaining - byte_size(data), new_acc, timeout)

      {:error, reason} ->
        {:error, reason}
    end
  end
end
