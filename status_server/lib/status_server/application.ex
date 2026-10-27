defmodule StatusServer.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    port = System.get_env("PORT", "4040") |> String.to_integer()

    children = [
      {Registry, keys: :duplicate, name: StatusServer.PubSub},
      StatusServer.ServerManager,
      {Bandit, plug: StatusServer.Router, port: port, ip: {0, 0, 0, 0}}
    ]

    opts = [strategy: :one_for_one, name: StatusServer.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
