using AKL.Common;
using AKL.Common.Util;

namespace AKL.Cli;

/// <summary>
///     Configures and starts the underlying virtual layer. Also updates it 
///     whenever the configuration gets modified if live-reload is activated.
/// </summary>
public class Application
{

    private readonly VirtualLayer virtualLayer;
    private readonly Autostart autostart;
    private readonly FileSystemWatcher? watcher;

    private DateTime lastChange = DateTime.MinValue;

    public Application(AklConfiguration configuration, FileSystemWatcher? watcher)
    {
        this.autostart = Autostart.WithAppName("another-keyboard-layer")
            .WithCmdArguments("--hide-window")
            .Build();
        this.watcher = watcher;

        if (watcher != null)
            watcher.Changed += (_, changeEvent) =>
            {
                if (changeEvent.ChangeType != WatcherChangeTypes.Changed)
                    return;

                DateTime lastWriteTime = File.GetLastWriteTime(changeEvent.FullPath);

                if (lastWriteTime == lastChange)
                {
                    return;
                }

                lastChange = lastWriteTime;
                Console.WriteLine("Trying to reload config file.");

                try
                {
                    Update(changeEvent.FullPath);
                }
                catch (Exception exception)
                {
                    ColorPrinter.WriteError("Failed: " + exception.Message);
                }
            };

        virtualLayer = new VirtualLayer(configuration);
    }

    private void Update(string path)
    {
        while (true)
        {
            try
            {
                var newConfiguration = AklConfiguration.FromString(
                    File.ReadAllText(path)
                );

                virtualLayer.Configuration = newConfiguration;
                virtualLayer.Update();
                UpdateAutostart();
                ColorPrinter.WriteSuccessful("Reload successful.");
                break;
            }
            catch (IOException)
            {
                ColorPrinter.WriteError("File locked");
                Thread.Sleep(100);
            }
        }
    }

    private void UpdateAutostart()
    {
        if (!OperatingSystem.IsWindows())
            return;

        try
        {
            autostart.SetAutostart(this.virtualLayer.Configuration.Autostart);
        }
        catch (Exception exception)
        {
            ColorPrinter.WriteError("Couldn't update autostart settings: " + exception.Message);
        }
    }

    public void Run()
    {
        if (SingleInstanceChecker.IsOtherAlreadyRunning("akl-application"))
        {
            ColorPrinter.WriteError(
                "Another akl application is already running."
            );
            Environment.Exit(1);
            return;
        }

        virtualLayer.Update();
        UpdateAutostart();
        Console.WriteLine("Quit with Ctrl + C");

        while (true)
        {
            Console.ReadLine();
        }
    }

}
