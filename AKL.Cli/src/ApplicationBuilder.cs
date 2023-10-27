using AKL.Common;

namespace AKL.Cli;

/// <summary>
///     Takes care of setup for actually starting the application by loading
///     configuration files and creating a file watcher if needed.
/// 
///     See also visualization section of the cli in the README for the program
///     flow.
/// </summary>
public class ApplicationBuilder
{

    public static Application Build(FileInfo? configFile, bool watchConfig)
    {
        var safeConfigFile = configFile ?? GetDefaultConfig();

        if (configFile != null && !configFile.Exists)
        {
            ColorPrinter.WriteError("The config option only accepts existing files.");
            Environment.Exit(1);
        }

        AklConfigurationProvider? configurationProvider = null;

        try
        {
            configurationProvider = AklConfigurationProvider.LoadFromFile(safeConfigFile);

            if (!safeConfigFile.Exists)
                configurationProvider.SaveToFile();
        }
        catch (AklConfigurationParsingException exception)
        {
            ColorPrinter.WriteError($"Couldn't parse config file: {exception.Message}");
            Environment.Exit(1);
        }

        return new Application(
            configurationProvider.GetConfiguration(),
            SetupFileWatcher(watchConfig, safeConfigFile)
        );
    }

    private static FileInfo GetDefaultConfig()
    {
        var configDirectory = Environment.GetEnvironmentVariable("XDG_CONFIG_HOME");

        if (string.IsNullOrWhiteSpace(configDirectory))
            configDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        return new FileInfo(Path.Combine(
            configDirectory,
            "another-keyboard-layer.toml"
        ));
    }

    private static FileSystemWatcher? SetupFileWatcher(bool watchConfig, FileInfo configFile)
    {
        FileSystemWatcher? watcher = null;

        if (watchConfig)
        {
            watcher = new FileSystemWatcher(configFile.DirectoryName ?? ".")
            {
                EnableRaisingEvents = true,
                Filter = configFile.Name,
                NotifyFilter = NotifyFilters.LastWrite
            };
        }

        return watcher;
    }

}
