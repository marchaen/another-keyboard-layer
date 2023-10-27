namespace AKL.Common;

using AKL.Core;

public unsafe class VirtualLayer
{

    public AklConfiguration Configuration { get; set; }
    private AklContext* akl;

    public VirtualLayer(AklConfiguration configuration)
    {
        Configuration = configuration;
        akl = AklCoreNativeInterface.init();

        AppDomain.CurrentDomain.ProcessExit += (_, _) => this.Destroy();
    }

    public void Update()
    {
        if (akl == null)
            return;

        Stop();

        AklCoreNativeInterface.set_switch_key(akl, Configuration.SwitchKey.ToFfi());

        if (Configuration.DefaultCombination != null)
            AklCoreNativeInterface.set_default_combination(akl, Configuration.DefaultCombination.ToFfi());
        else
            AklCoreNativeInterface.set_default_combination(akl, new FfiKeyCombination());

        AklCoreNativeInterface.clear_mappings(akl);

        foreach (KeyValuePair<KeyCombination, KeyCombination> mapping in Configuration.Mappings)
        {
            // At this point no invalid key combination can exist so this method
            // should never cause an error.
            AklCoreNativeInterface.add_mapping(akl, mapping.Key.ToFfi(), mapping.Value.ToFfi());
        }

        AklCoreNativeInterface.start(akl);
    }

    public void Stop()
    {
        if (akl == null)
            return;

        if (AklCoreNativeInterface.is_running(akl))
        {
            var error = AklCoreNativeInterface.stop(akl);

            if (error.has_error)
            {
                AklCoreNativeInterface.destroy_error_message(error.error_message);
            }
        }
    }

    protected void Destroy()
    {
        if (akl != null)
        {
            Stop();
            AklCoreNativeInterface.destroy(akl);
            akl = null;
        }
    }

}
