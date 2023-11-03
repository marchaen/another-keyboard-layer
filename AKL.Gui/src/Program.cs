using Avalonia;
using System;

using AKL.Common;

namespace AKL.Gui;

class Program
{

    // Initialization code. Don't use any Avalonia, third-party APIs or any
    // SynchronizationContext-reliant code before AppMain is called: things aren't initialized
    // yet and stuff might break.
    [STAThread]
    public static void Main(string[] args)
    {
        // The console has to be hidden manually because setting the output type
        // to winexe in the csproj isn't supported when building from linux. 
        //
        // Related issue and question: 
        // https://github.com/dotnet/sdk/issues/3309
        // https://stackoverflow.com/questions/64588598/app-opens-console-window-when-being-build-with-docker
        ConsoleWindowHider.Execute();

        BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
    }

    // Avalonia configuration, don't remove; also used by visual designer.
    public static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace();

}
