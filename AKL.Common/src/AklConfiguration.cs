namespace AKL.Common;

using Tomlyn.Model;

public class AklConfiguration
{

    private TomlAklConfiguration origin;

    public bool Autostart { get; set; }

    public Key SwitchKey { get; set; }
    public KeyCombination? DefaultCombination { get; set; }

    public Dictionary<KeyCombination, KeyCombination> Mappings { get; set; } = new Dictionary<KeyCombination, KeyCombination>();

    public static AklConfiguration FromString(string raw)
    {
        return new AklConfiguration(new TomlAklConfiguration());
    }

    private AklConfiguration(TomlAklConfiguration origin)
    {
        this.origin = origin;
        SwitchKey = new Key(null, null, KeyKind.Text);
    }

    public override string ToString()
    {
        return "";
    }

}

internal class TomlAklConfiguration : ITomlMetadataProvider
{

    public bool StartWithSystem { get; set; }
    public string? SwitchKey { get; set; }
    public string? DefaultSimulationCombination { get; set; }
    public Dictionary<string, string>? Mappings { get; set; }

    // Storage for comments in the configuration file so that they can be saved
    // back to file when the in memory configuration gets updated.
    TomlPropertiesMetadata? ITomlMetadataProvider.PropertiesMetadata { get; set; }

}
