namespace AKL.Common;

using AKL.Core;

/// <summary>
///     Safe and c# idiomatic wrapper around the core system lib.
///     
///     When disposed of takes care of safely disposing of the associated system
///     resources before the program exists.
/// 
///     <seealso cref="AklConfiguration"/>
/// </summary>
public unsafe class VirtualLayer
{

    public AklConfiguration Configuration { get; set; }

    private AklContext* akl;

    /// <summary>
    ///     Initializes the akl context associated with this virtual layer call
    ///     <see cref="Update"/> to configure and start the underlying native
    ///     akl context.
    ///
    ///     Also registers an 
    ///     <see cref="AppDomain.CurrentDomain.ProcessExit">process exit</see>
    ///     handler for cleaning up itself.
    /// </summary>
    /// <param name="configuration">
    ///     Configures the switch key, default key combination, mappings, etc.
    /// </param>
    public VirtualLayer(AklConfiguration configuration)
    {
        Configuration = configuration;
        akl = AklCoreNativeInterface.init();
        AppDomain.CurrentDomain.ProcessExit += (_, _) => this.Destroy();
    }

    /// <summary>
    ///     Updates the configuration of the native akl context with the
    ///     according to this wrapper then restarts it.
    /// </summary>
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

    /// <summary>
    ///     Stops the native virtual layer if it is running but doesn't clean up
    ///     any associated resources.
    ///     
    ///     Calling <see cref="Update"/> after stop is completely fine and works
    ///     without any unexpected behavior.
    /// </summary>
    public void Stop()
    {
        if (akl == null)
            return;

        if (AklCoreNativeInterface.is_running(akl))
        {
            var error = AklCoreNativeInterface.stop(akl);

            if (error.has_error)
                AklCoreNativeInterface.destroy_error_message(error.error_message);
        }
    }

    // Internal method used to safely clean up all resources associated with the
    // native library.
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
