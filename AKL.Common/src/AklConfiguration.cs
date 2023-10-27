namespace AKL.Common;

using Tomlyn;
using Tomlyn.Model;

// TODO: Write documentation for this exception
public class AklConfigurationParsingException : Exception
{
    public AklConfigurationParsingException(string? message) : base(message) { }
}

public class AklConfiguration
{

    private TomlAklConfiguration origin;

    public bool Autostart { get; set; }

    public Key SwitchKey { get; set; }
    public KeyCombination? DefaultCombination { get; set; }

    public Dictionary<KeyCombination, KeyCombination> Mappings { get; set; } = new Dictionary<KeyCombination, KeyCombination>();

    // TODO: Write documentation
    public static AklConfiguration FromString(string raw)
    {
        TomlAklConfiguration model;

        try
        {
            model = Toml.ToModel<TomlAklConfiguration>(raw);
        }
        catch (TomlException exception)
        {
            throw new AklConfigurationParsingException("Can't parse toml akl configuration: " + exception.Message);
        }

        AklConfiguration configuration;

        try
        {
            configuration = new AklConfiguration(model);
        }
        catch (Exception exception)
        {
            throw new AklConfigurationParsingException("Specified configuration value is invalid: " + exception.Message);
        }

        return configuration;
    }

    // TODO: Write documentation
    private AklConfiguration(TomlAklConfiguration origin)
    {
        this.origin = origin;

        if (origin.StartWithSystem == null)
            throw new AklConfigurationParsingException("No start with system key in configuration file.");

        if (origin.SwitchKey == null)
            throw new AklConfigurationParsingException("No switch key in configuration file.");

        if (origin.DefaultSimulationCombination == null)
            throw new AklConfigurationParsingException("No default combination key in configuration file.");

        if (origin.Mappings == null)
            throw new AklConfigurationParsingException("No mappings table in configuration file.");

        Autostart = origin.StartWithSystem ?? false;
        SwitchKey = Key.TryParse(origin.SwitchKey);

        if (origin.DefaultSimulationCombination == "")
        {
            DefaultCombination = null;
        }
        else
        {
            DefaultCombination = KeyCombination.TryParse(origin.DefaultSimulationCombination ?? "Can't be null!");
        }

        Mappings = origin.Mappings.ToDictionary((kvp) => KeyCombination.TryParse(kvp.Key), (kvp) => KeyCombination.TryParse(kvp.Value));
    }

    public override bool Equals(object? obj)
    {
        if (obj == null || GetType() != obj.GetType())
        {
            return false;
        }

        AklConfiguration other = (AklConfiguration)obj;

        if (this.DefaultCombination != null)
        {
            if (!this.DefaultCombination.Equals(other.DefaultCombination))
                return false;
        }
        else if (other.DefaultCombination != null)
        {
            return false;
        }

        bool mappingsEqual =
            this.Mappings.Keys.Count == other.Mappings.Keys.Count &&
            this.Mappings.Keys.All(
                key => other.Mappings.ContainsKey(key)
                    && this.Mappings[key].Equals(other.Mappings[key])
            );

        return this.Autostart == other.Autostart &&
            this.SwitchKey.Equals(other.SwitchKey) &&
            mappingsEqual;
    }

    public override int GetHashCode()
    {
        unchecked
        {
            int hashcode = this.Mappings.Aggregate(1430287,
                (hash, kvp) => hash ^ (kvp.Key, kvp.Value).GetHashCode()
            );
            return hashcode * 7302013 ^ (this.Autostart, this.SwitchKey, this.DefaultCombination).GetHashCode();
        }
    }

    public override string ToString()
    {
        origin.StartWithSystem = this.Autostart;
        origin.SwitchKey = this.SwitchKey.ToString();
        origin.DefaultSimulationCombination = this.DefaultCombination?.ToString();
        origin.Mappings = this.Mappings.ToDictionary((kvp) => kvp.Key.ToString() ?? "", (kvp) => kvp.Value.ToString() ?? "");

        return Toml.FromModel(origin);
    }

}

internal class TomlAklConfiguration : ITomlMetadataProvider
{

    public bool? StartWithSystem { get; set; }
    public string? SwitchKey { get; set; }
    public string? DefaultSimulationCombination { get; set; }
    public Dictionary<string, string>? Mappings { get; set; }

    // Storage for comments in the configuration file so that they can be saved
    // back to file when the in memory configuration gets updated.
    TomlPropertiesMetadata? ITomlMetadataProvider.PropertiesMetadata { get; set; }

}
