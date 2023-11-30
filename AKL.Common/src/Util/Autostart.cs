namespace AKL.Common.Util;

using System.Runtime.Versioning;

/// <summary>
///     Provides a simple api to enable or disable autostart of the provided
///     custom command or executable with a specified name.
///
///     Use <see cref="Autostart.WithAppName(string)"/> to create a builder for
///     an autostart item.
/// </summary>
public class Autostart
{

    public class AutostartBuilder
    {

        private string name;
        private string? executable;
        private string? arguments;

        public AutostartBuilder(string name)
        {
            this.name = name;
        }

        public AutostartBuilder WithCmdArguments(string arguments)
        {
            this.arguments = arguments;
            return this;
        }

        public AutostartBuilder WithTargetExecutable(string executable)
        {
            this.executable = executable;
            return this;
        }

        /// <summary>
        ///     Builds an autostart item corresponding to the specified
        ///     parameters.
        ///
        ///     If the cmd arguments weren't specified no arguments will be
        ///     provided to the executable.
        ///
        ///     If the target executable wasn't specified the currently
        ///     executing binary will be used instead.
        ///
        ///     The command will be prefixed with "conhost.exe " to force a
        ///     compatible environment when the command is actually run. This is
        ///     needed because the default cmd host for cli programs was
        ///     <see href="https://devblogs.microsoft.com/commandline/windows-terminal-is-now-the-default-in-windows-11/"> 
        ///         changed
        ///     </see>
        ///     in windows 11.
        /// </summary>
        public Autostart Build()
        {
            this.executable ??= $"{AppDomain.CurrentDomain.BaseDirectory}{AppDomain.CurrentDomain.FriendlyName}.exe";
            this.arguments ??= "";

            return new Autostart(name, $"conhost.exe {executable} {arguments}");
        }

    }

    public static AutostartBuilder WithAppName(string name)
    {
        return new AutostartBuilder(name);
    }

    private string name;
    private string command;

    /// <summary>
    ///     Creates an autostart item for an app with the specified name and
    ///     command.
    ///
    ///     Use <see cref="Autostart.WithAppName(string)"/> for an easier way
    ///     to specify the cmd / arguments and or automatically find the
    ///     location of the currently executing binary.
    /// </summary>
    /// <param name="name">
    ///     The name or identifier of this autostart item which will be used to
    ///     access the registry.
    /// </param>
    /// <param name="command">
    ///     The command which should be executed on startup.
    /// </param>
    public Autostart(string name, string command)
    {
        this.name = name;
        this.command = command;
    }

    /// <summary>
    ///     Calls <see cref="Enable()"/> if the enabled parameter is <c>true</c>
    ///     and otherwise <see cref="Disable()"/>.
    ///
    ///     Note: This method could throw if the autostart registry key doesn't
    ///     exist or the user doesn't have permission to write to the registry.
    /// </summary>
    /// <param name="enabled">Enable or disable autostart for this app.</param>
    [SupportedOSPlatform("windows")]
    public void SetAutostart(bool enabled)
    {
        if (enabled)
            this.Enable();
        else
            this.Disable();
    }

    /// <summary>
    ///     Enables autostart for this app by setting a specific value in the
    ///     registry to <see cref="this.command"/>.
    ///
    ///     Note: This method could throw if the autostart registry key doesn't
    ///     exist or the user doesn't have permission to write to the registry.
    /// </summary>
    /// <seealso cref="OpenRegistry()"/>
    [SupportedOSPlatform("windows")]
    public void Enable()
    {
        OpenRegistry().SetValue(this.name, this.command);
    }

    /// <summary>
    ///     Disables autostart for this app by deleting a specific value in the
    ///     registry.
    ///
    ///     Note: This method could throw if the autostart registry key doesn't
    ///     exist or the user doesn't have permission to write to the registry.
    /// </summary>
    /// <seealso cref="OpenRegistry()"/>
    [SupportedOSPlatform("windows")]
    [SupportedOSPlatform("windows")]
    public void Disable()
    {
        OpenRegistry().DeleteValue(this.name);
    }

    /// <summary>
    ///     Opens or creates the
    ///     <see href="https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys">
    ///         Run Registry Key
    ///     </see>
    ///     for the currently logged in user.
    ///
    ///     This operation uses
    ///     <see cref="Microsoft.Win32.RegistryKey.CreateSubKey(string, bool)"/>
    ///     directly and doesn't handle any exceptions that it could throw e. g.
    ///     if the user doesn't have permission to create / write or read the
    ///     run registry key.
    /// </summary>
    /// <returns>The registry key if it could be opened.</returns>
    [SupportedOSPlatform("windows")]
    private Microsoft.Win32.RegistryKey OpenRegistry()
    {
        // See https://stackoverflow.com/a/675347
        Microsoft.Win32.RegistryKey? autostart;

        // Creates or **opens** the autostart registry key if it doesn't exist.
        autostart = Microsoft.Win32.Registry.CurrentUser.CreateSubKey(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", true
        );

        if (autostart == null)
            throw new Exception("Failed to open autostart registry key.");

        return autostart;
    }

}
