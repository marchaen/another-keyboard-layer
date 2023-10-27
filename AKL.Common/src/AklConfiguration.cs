namespace AKL.Common;

using Tomlyn;
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
        return new AklConfiguration(Toml.ToModel<TomlAklConfiguration>(raw));
    }

    private AklConfiguration(TomlAklConfiguration origin)
    {
        this.origin = origin;

        if (origin.Mappings == null)
            throw new ArgumentException("No mappings table in configuration file.");

        Autostart = Boolean.Parse(origin.StartWithSystem ?? "No start with system in configuration file.");
        SwitchKey = Key.TryParse(origin.SwitchKey ?? "No switch key in configuration file.");

        if (origin.DefaultSimulationCombination == "")
        {
            DefaultCombination = null;
        }
        else
        {
            DefaultCombination = KeyCombination.TryParse(origin.DefaultSimulationCombination ?? "No default combination in configuration file.");
        }

        Mappings = origin.Mappings.ToDictionary((kvp) => KeyCombination.TryParse(kvp.Key), (kvp) => KeyCombination.TryParse(kvp.Value));
    }

    public override string ToString()
    {
        origin.StartWithSystem = this.Autostart.ToString();
        origin.SwitchKey = this.SwitchKey.ToString();
        origin.DefaultSimulationCombination = this.DefaultCombination?.ToString();
        origin.Mappings = this.Mappings.ToDictionary((kvp) => kvp.Key.ToString() ?? "", (kvp) => kvp.Value.ToString() ?? "");

        return Toml.FromModel(origin);
    }

}

internal class TomlAklConfiguration : ITomlMetadataProvider
{

    public string? StartWithSystem { get; set; }
    public string? SwitchKey { get; set; }
    public string? DefaultSimulationCombination { get; set; }
    public Dictionary<string, string>? Mappings { get; set; }

    // Storage for comments in the configuration file so that they can be saved
    // back to file when the in memory configuration gets updated.
    TomlPropertiesMetadata? ITomlMetadataProvider.PropertiesMetadata { get; set; }

}
