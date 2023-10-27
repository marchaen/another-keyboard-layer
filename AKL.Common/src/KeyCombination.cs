namespace AKL.Common;

public class KeyCombination
{

    public static char KEY_SEPARATOR = '+';

    private Key[] keys;

    public KeyCombination(Key[] keys)
    {
        if (keys.Length > 4)
            throw new ArgumentException("Each key combination can only be four keys long.");

        this.keys = new Key[keys.Length];

        for (var i = 0; i < keys.Length; i++)
        {
            this.keys[i] = keys[i];
        }
    }

    /// <summary>
    ///     Tries to parse raw as a keyboard combination.
    /// </summary>
    /// <param name="raw">
    ///     A raw keyboard combination is multiple <see cref="Key">keys</see> 
    ///     (4 maximum) separated by <see cref="KEY_SEPARATOR"/>.
    /// </param>
    /// <exception cref="ArgumentException">
    ///     If no virtual key code with the specified name could be found.
    /// </exception>
    /// <returns>
    ///     A key combination which is used to configure the virtual layer.
    /// </returns>
    public static KeyCombination TryParse(string raw)
    {
        if (String.IsNullOrEmpty(raw.Trim()))
            throw new ArgumentException("Raw can't be empty.");

        var rawKeys = raw.Split(KEY_SEPARATOR);

        if (rawKeys.Length > 4)
            throw new ArgumentException("Only four keys for each combination can be specified.");

        var keys = rawKeys.Select(Key.TryParse);

        if (keys.Count() != keys.Distinct().Count())
            throw new ArgumentException("Each key can only be used once in each combination.");

        return new KeyCombination(keys.ToArray());
    }

    public override string ToString()
    {
        return String.Join(KEY_SEPARATOR, keys.AsEnumerable());
    }

    public override bool Equals(object? obj)
    {
        if (obj == null || GetType() != obj.GetType()) return false;

        KeyCombination other = (KeyCombination)obj;

        return this.GetHashCode() == other.GetHashCode();
    }

    public override int GetHashCode()
    {
        return keys.Select((key) => key.GetHashCode()).Sum();
    }

}
