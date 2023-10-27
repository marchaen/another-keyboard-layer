namespace AKL.Common;

public class VirtualLayer
{

    private AklConfiguration configuration;

    public bool Autostart { get; set; }

    public Key SwitchKey { get; set; }
    public KeyCombination? DefaultCombination { get; set; }

    public Dictionary<KeyCombination, KeyCombination> Mappings { get; set; } = new Dictionary<KeyCombination, KeyCombination>();

    public VirtualLayer(AklConfiguration configuration, Key switchKey)
    {
        this.configuration = configuration;
        SwitchKey = switchKey;
    }

    public void Update()
    {
        // TODO: Update native library with current state
    }

}
