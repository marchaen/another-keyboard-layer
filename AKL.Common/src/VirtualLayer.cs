namespace AKL.Common;

public class VirtualLayer
{

    public AklConfiguration Configuration { get; set; }

    public VirtualLayer(AklConfiguration configuration)
    {
        Configuration = configuration;
    }

    public void Update()
    {
        // TODO: Update native library with current state
    }

}
