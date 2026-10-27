# IO Configuration

Configuration file located at `~/.config/sqrilizz/io.json` (or OS equivalent) controls low-level file and networking operations.

## Properties

### `transfer_limit` - `number`
The number of concurrent tasks that should be used to download files (addons, assets, libraries). Defaults to optimal value for your system.

### `link_method` - `string`, `"hard" | "soft" | "copy"`
The IO method used to link shared files on the filesystem. `hard` is used by default as it is the most performant and compatible, but `soft` can be used if you are linking between different filesystems.

### `data_path` - `string`
Path to the data folder, containing instances, packages, and internal sqrilizz data. Can be used to save filesystem space by changing where sqrilizz stores files.

### `config_path` - `string`
Path to the config folder, containing sqrilizz configuration.

### `cli_icons` - `boolean`
Enables or disables icons for the CLI. Defaults to false.

### `cli_wrap` - `boolean`
Enables or disables text wrapping for the CLI. Defaults to true.
