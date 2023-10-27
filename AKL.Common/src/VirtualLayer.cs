namespace AKL.Common;

public class VirtualLayer
{

    private AklConfigurationProvider configurationProvider;
    public AklConfiguration Configuration { get; set; }

    public VirtualLayer(AklConfigurationProvider configurationProvider)
    {
        this.configurationProvider = configurationProvider;
        Configuration = configurationProvider.GetConfiguration();
    }

    public void Update()
    {
        // TODO: Update native library with current state
    }

}
