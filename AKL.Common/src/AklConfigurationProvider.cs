namespace AKL.Common;

using System.Reflection;

public class AklConfigurationProvider
{

    private readonly FileInfo file;
    private readonly AklConfiguration configuration;

    public FileInfo ConfigFile { get => this.file; }

    /// <summary>
    ///     Creates a configuration provider with the parsed and loaded
    ///     configuration which the file specifies.
    ///     
    ///     If the specified file doesn't exist the default configuration is
    ///     going to be loaded instead. If the loading is successful the default
    ///     location will be saved to the specified path.
    /// </summary>
    /// <param name="file">
    ///     The file which should be loaded and also the location where 
    ///     <see cref="AklConfigurationProvider.SaveToFile"/> will save the 
    ///     serialized configuration.
    /// </param>
    /// <returns>
    ///     A configuration that can be used by the <see cref="VirtualLayer"/>.
    /// </returns>
    /// <exception cref="AklConfigurationParsingException">
    ///     If anything goes wrong in the deserialization or parsing step of the
    ///     <see cref="AklConfiguration.FromString(string)"> method.
    /// </exception>
    public static AklConfigurationProvider LoadFromFile(FileInfo file)
    {
        var path = file.FullName;

        if (!file.Exists)
        {
            var assembly = Assembly.GetEntryAssembly();
            var location = assembly?.Location;

            if (assembly != null && !string.IsNullOrEmpty(location))
                path = new FileInfo(location).FullName ?? ".";
            else
                path = ".";

            path = Path.Combine(path, "default-config.toml");
        }

        var provider = new AklConfigurationProvider(file, AklConfiguration.FromString(File.ReadAllText(path)));

        if (!file.Exists)
            provider.SaveToFile();

        return provider;
    }

    /// <summary>
    ///     Creates a configuration provider with the parsed and loaded
    ///     configuration at the default storage location.
    ///
    ///     The default storage location is
    ///     $XDG_CONFIG_HOME/another-keyboard-layer.toml if xdg-variables are
    ///     set, otherwise <see cref="Environment.SpecialFolder.UserProfile"/>
    ///     and <c>.config/another-keyboard-layer.toml</c> are combined to form
    ///     the final default location.
    /// </summary>
    /// <returns>
    ///     A configuration that can be used by the <see cref="VirtualLayer"/>.
    /// </returns>
    /// <exception cref="AklConfigurationParsingException">
    ///     If anything goes wrong in the deserialization or parsing step of the
    ///     <see cref="AklConfiguration.FromString(string)"> method.
    /// </exception>
    /// <seealso cref="AklConfigurationProvider.LoadFromFile(FileInfo)"/>
    public static AklConfigurationProvider LoadFromDefaultLocation()
    {
        var configDirectory = Environment.GetEnvironmentVariable("XDG_CONFIG_HOME");

        if (string.IsNullOrWhiteSpace(configDirectory))
            configDirectory = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
                ".config"
            );

        return LoadFromFile(new FileInfo(Path.Combine(
            configDirectory,
            "another-keyboard-layer.toml"
        )));
    }

    private AklConfigurationProvider(FileInfo file, AklConfiguration configuration)
    {
        this.file = file;
        this.configuration = configuration;
    }

    public AklConfiguration GetConfiguration()
    {
        return this.configuration;
    }

    /// <summary>
    ///     Saves the current state of the configuration to the file which is
    ///     set in <see cref="AklConfigurationProvider.LoadFromFile(FileInfo)"/>.
    /// </summary>
    public void SaveToFile()
    {
        File.WriteAllText(this.file.FullName, this.configuration.ToString());
    }

}
