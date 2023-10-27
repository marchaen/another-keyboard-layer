namespace AKL.Common;

using System.Reflection;

public class AklConfigurationProvider
{

    private readonly FileInfo file;
    private readonly AklConfiguration configuration;

    /// <summary>
    ///     Creates a configuration provider with the parsed and loaded
    ///     configuration which the file specifies.
    ///     
    ///     If the specified file doesn't exist the default configuration is
    ///     going to be loaded instead.
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

        return new AklConfigurationProvider(file, AklConfiguration.FromString(File.ReadAllText(path)));
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
