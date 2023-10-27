using AKL.Common;

namespace AKL.Cli;

/// <summary>
///     Configures and starts the underlying virtual layer. Also updates it 
///     whenever the configuration gets modified if live-reload is activated.
/// </summary>
public class Application
{

    private readonly VirtualLayer virtualLayer;
    private readonly FileSystemWatcher? watcher;

    public Application(AklConfiguration configuration, FileSystemWatcher? watcher)
    {
        this.watcher = watcher;

        if (watcher != null)
            watcher.Changed += (_, changeEvent) =>
            {
                if (changeEvent.ChangeType != WatcherChangeTypes.Changed)
                    return;

                Console.WriteLine("Trying to reload config file.");

                try
                {
                    var newConfiguration = AklConfiguration.FromString(
                        File.ReadAllText(changeEvent.FullPath)
                    );

                    Update(newConfiguration);

                    ColorPrinter.WriteSucessful("Reload successful.");
                }
                catch (Exception exception)
                {
                    ColorPrinter.WriteError("Failed: " + exception.Message);
                }
            };

        virtualLayer = new VirtualLayer(configuration);
        Update(configuration);
    }

    private void Update(AklConfiguration configuration)
    {
        virtualLayer.Update();
    }

    public void Run()
    {
        Console.WriteLine("Quit with Ctrl + C");

        while (true)
        {
            Console.ReadLine();
        }
    }

}
