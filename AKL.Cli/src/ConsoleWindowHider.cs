namespace AKL.Cli;

using System.Runtime.InteropServices;

// See https://stackoverflow.com/a/3571628
public class ConsoleWindowHider {

    // https://learn.microsoft.com/en-us/windows/console/getconsolewindow
    [DllImport("kernel32.dll")]
    static extern IntPtr GetConsoleWindow();

    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    const int SW_HIDE = 0;

    public static void Execute() {
        ShowWindow(GetConsoleWindow(), SW_HIDE);
    }

}
