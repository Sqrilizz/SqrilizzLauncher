defmodule StatusServerTest do
  use ExUnit.Case

  alias StatusServer.Minecraft.Ping

  test "encodes single-byte varint" do
    assert Ping.encode_varint(0) == <<0>>
    assert Ping.encode_varint(1) == <<1>>
    assert Ping.encode_varint(127) == <<127>>
  end

  test "encodes multi-byte varint" do
    assert Ping.encode_varint(128) == <<0x80, 0x01>>
    assert Ping.encode_varint(255) == <<0xFF, 0x01>>
    assert Ping.encode_varint(2147483647) == <<0xFF, 0xFF, 0xFF, 0xFF, 0x07>>
  end

  test "lists servers" do
    servers = StatusServer.list_servers()
    assert is_list(servers)
    assert length(servers) >= 1
  end

  test "adds and removes a server" do
    {:ok, added} = StatusServer.add_server(%{"id" => "test_server", "name" => "Test", "host" => "127.0.0.1", "port" => 25565})
    assert added.id == "test_server"

    servers = StatusServer.list_servers()
    assert Enum.any?(servers, &(&1.id == "test_server"))

    StatusServer.remove_server("test_server")
    remaining = StatusServer.list_servers()
    refute Enum.any?(remaining, &(&1.id == "test_server"))
  end
end
