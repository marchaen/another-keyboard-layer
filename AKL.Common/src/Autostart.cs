namespace AKL.Common;

using System.Runtime.Versioning;

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

    public Autostart(string name, string command)
    {
        this.name = name;
        this.command = command;
    }

    [SupportedOSPlatform("windows")]
    public void SetAutostart(bool enabled)
    {
        if (enabled)
            this.Enable();
        else
            this.Disable();
    }

    [SupportedOSPlatform("windows")]
    public void Enable()
    {
        OpenRegistry().SetValue(this.name, this.command);
    }

    [SupportedOSPlatform("windows")]
    public void Disable()
    {
        OpenRegistry().DeleteValue(this.name);
    }

    [SupportedOSPlatform("windows")]
    private Microsoft.Win32.RegistryKey OpenRegistry()
    {
        // See https://stackoverflow.com/a/675347
        Microsoft.Win32.RegistryKey? autostart;

        autostart = Microsoft.Win32.Registry.CurrentUser.OpenSubKey(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", true
        );

        if (autostart == null)
            throw new Exception("Failed to open autostart registry key.");

        return autostart;
    }

}
