defmodule StatusServer.MixProject do
  use Mix.Project

  def project do
    [
      app: :status_server,
      version: "0.1.0",
      elixir: "~> 1.20",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {StatusServer.Application, []}
    ]
  end

  defp deps do
    [
      {:bandit, "~> 1.6"},
      {:plug, "~> 1.16"},
      {:websock_adapter, "~> 0.5"},
      {:jason, "~> 1.4"}
    ]
  end
end
