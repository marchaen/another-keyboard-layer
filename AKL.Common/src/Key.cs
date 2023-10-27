namespace AKL.Common;

public class Key
{

    private SpecialKey? specialKey;
    private char? unicodeKey;
    private KeyKind kind;

    public static Key TryParse(string raw)
    {
        // TODO: Think about parsing algorithm and implement it.
        return new Key();
    }

}

internal enum KeyKind
{
    Simple,
    Special
}

// TODO: Find resource that lists all special keys and add them here
internal enum SpecialKey
{
    Ctrl,
    Shift,
    Space,
}
