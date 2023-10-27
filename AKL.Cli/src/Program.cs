using AKL.Common;

Console.WriteLine("Simple interactive key combination parser! [Ctrl + C to quit]");

string? rawKeyCombination;
KeyCombination result;

while (true)
{
    rawKeyCombination = Console.ReadLine();

    if (rawKeyCombination == null)
    {
        Console.WriteLine("Bye");
        break;
    }

    try
    {
        result = KeyCombination.TryParse(rawKeyCombination);
    }
    catch (ArgumentException exception)
    {
        Console.WriteLine($"Unfortunately that input wasn't a valid key combination: {exception.Message}");
        continue;
    }

    Console.WriteLine($"You typed the key combination: {result.ToString()} ({result.GetHashCode()})");
}
