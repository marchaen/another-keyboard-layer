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
    }

    public void Update()
    {
        if (AklCoreNativeInterface.is_running(akl))
        {
            var error = AklCoreNativeInterface.stop(akl);

            if (error.has_error)
            {
                AklCoreNativeInterface.destroy_error_message(error.error_message);
            }
        }

        AklCoreNativeInterface.set_switch_key(akl, Configuration.SwitchKey.ToFfi());

        if (Configuration.DefaultCombination != null)
            AklCoreNativeInterface.set_default_combination(akl, Configuration.DefaultCombination.ToFfi());

        AklCoreNativeInterface.clear_mappings(akl);

        foreach (KeyValuePair<KeyCombination, KeyCombination> mapping in Configuration.Mappings)
        {
            AklCoreNativeInterface.add_mapping(akl, mapping.Key.ToFfi(), mapping.Value.ToFfi());
        }

        AklCoreNativeInterface.start(akl);
    }

}
