using System.CommandLine;

var configFileOption = new Option<FileInfo?>("--config");
configFileOption.AddAlias("-c");

configFileOption.Description =
@"Explicitly specify a configuration file to load. The program will exit with a
non zero code if the file doesn't exist.

By default the configuration file will be stored at
$XDG_CONFIG_HOME/another-keyboard-layer.toml. If $XDG_CONFIG_HOME is not set
$HOME/.config is used instead.";

var liveReloadOption = new Option<bool>(name: "--live-reload");
liveReloadOption.AddAlias("-l");

liveReloadOption.Description =
@"Reload another keyboard layer when the configuration file changes. Respects 
overriding the default config path with the --config option.";

var command = new RootCommand("run another keyboard layer from the terminal")
{
    configFileOption,
    liveReloadOption
};

command.SetHandler(
    (configFile, liveReload) =>
        Console.WriteLine("TODO: Write application wrapper"),
    configFileOption, liveReloadOption
);

command.Invoke(args);
