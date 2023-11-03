namespace AKL.Cli;

using System.Runtime.InteropServices;

/// <summary>
///     Provides a single method to hide the console window.
/// </summary>
/// <seealso cref="Execute()"/>
public class ConsoleWindowHider
{

    // https://learn.microsoft.com/en-us/windows/console/getconsolewindow
    [DllImport("kernel32.dll")]
    static extern IntPtr GetConsoleWindow();

    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    const int SW_HIDE = 0;

    /// <summary>
    ///     Hides the console of this application if it is currently visible.
    /// /// </summary>
    /// <seealso href="https://stackoverflow.com/a/3571628"/>
    public static void Execute()
    {
        ShowWindow(GetConsoleWindow(), SW_HIDE);
    }

}
