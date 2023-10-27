namespace AKL.Common;

public class AklConfigurationProvider
{

    private FileInfo file;
    private AklConfiguration configuration;

    public static AklConfigurationProvider ReadFromFile(FileInfo file)
    {
        // TODO: Implement configuration parsing
        return new AklConfigurationProvider(file, AklConfiguration.FromString(""));
    }

    public static AklConfigurationProvider WithSaveDefault(FileInfo output)
    {
        // TODO: Implement default configuration parsing and saving to the
        // output file
        return new AklConfigurationProvider(output, AklConfiguration.FromString(""));
    }

    private AklConfigurationProvider(FileInfo file, AklConfiguration configuration)
    {
        this.file = file;
        this.configuration = configuration;
    }

    internal AklConfiguration GetConfiguration()
    {
        return this.configuration;
    }

    public void SaveToFile()
    {
        // TODO: Implement toml to file configuration
    }

}
