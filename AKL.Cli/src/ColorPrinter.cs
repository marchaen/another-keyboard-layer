namespace AKL.Cli;

/// <summary>
///     Utility for printing messages which a specific meaning to the console
///     while also taking care of clean up and deciding the highlighting.
/// </summary>
public class ColorPrinter
{

    public static void WriteSucessful(string message)
    {
        WriteWithColor(ConsoleColor.Green, message);
    }

    public static void WriteError(string message)
    {
        WriteWithColor(ConsoleColor.Red, message);
    }

    private static void WriteWithColor(ConsoleColor color, string message)
    {
        Console.ForegroundColor = color;
        Console.Error.WriteLine(message);
        Console.ResetColor();
    }

}
