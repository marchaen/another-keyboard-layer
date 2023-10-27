using System.CommandLine;

using AKL.Cli;

// Cli argument parsing

var configFileOption = new Option<FileInfo?>("--config");
configFileOption.AddAlias("-c");

configFileOption.Description =
@"Explicitly specify a configuration file to load. The program will exit with a
non zero code if the file doesn't exist.

By default the configuration file will be stored at
$XDG_CONFIG_HOME/another-keyboard-layer.toml. If $XDG_CONFIG_HOME is not set
$HOME/.config is used instead.";

var liveReloadOption = new Option<bool>("--live-reload");
liveReloadOption.AddAlias("-l");

liveReloadOption.Description =
@"Reload another keyboard layer when the configuration file changes. Respects 
overriding the default config path with the --config option.";

var hideWindowOption = new Option<bool>("--hide-window");

hideWindowOption.Description =
@"Hides the console window this application was started in. Used for autostart.";

// Command definition

var command = new RootCommand("Run another keyboard layer from the terminal.")
{
    configFileOption,
    liveReloadOption,
    hideWindowOption,
};

// Executing the command

command.SetHandler(
    (configFile, liveReload, hideWindow) =>
        ApplicationBuilder.Build(configFile, liveReload, hideWindow).Run(),
    configFileOption, liveReloadOption, hideWindowOption
);

command.Invoke(args);
