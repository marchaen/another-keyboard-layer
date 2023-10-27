namespace AKL.Common;

public class AklConfiguration
{

    private FileInfo file;
    // TODO: Add toml library to project and correct field in this class

    private VirtualLayer? layer;

    private AklConfiguration(FileInfo file)
    {
        this.file = file;
    }

    public static AklConfiguration ReadFromFile(FileInfo file)
    {
        // TODO: Implement configuration parsing
        return new AklConfiguration(file);
    }

    public static AklConfiguration WithSaveDefault(FileInfo output)
    {
        // TODO: Implement default configuration parsing and saving to the
        // output file
        return new AklConfiguration(output);
    }

    public VirtualLayer TryParse()
    {
        // TODO: Create a new virtual layer from the parsed raw config.
        return new VirtualLayer(this, new Key());
    }

    public void SaveToFile()
    {
        // TODO: Implement toml to file configuration
    }

}
