namespace AKL.Common.Tests;

[TestClass]
public class KeyTests
{

    [TestMethod]
    public void ParseVirtualKeyCodeFromString()
    {
        Assert.AreEqual(VirtualKeyCode.Back, VirtualKeyCodeParser.Parse("Back"));
        Assert.IsNull(VirtualKeyCodeParser.Parse("Something"));
    }

    [TestMethod]
    public void ParseKeyFromString()
    {
        var expectedKey = new Key(VirtualKeyCode.Back, null, KeyKind.Virtual);
        var parsedKey = Key.TryParse("Back");

        Assert.AreEqual(expectedKey, parsedKey);
        Assert.AreEqual(expectedKey.GetHashCode(), parsedKey.GetHashCode());

        expectedKey = new Key(null, 'a', KeyKind.Text);
        parsedKey = Key.TryParse("a");

        Assert.AreEqual(expectedKey, parsedKey);
        Assert.AreEqual(expectedKey.GetHashCode(), parsedKey.GetHashCode());

        Assert.ThrowsException<ArgumentException>(() => Key.TryParse("With Whitespace"));
        Assert.ThrowsException<ArgumentException>(() => Key.TryParse("This key doesn't exist"));
    }

    [TestMethod]
    public void ParseKeyCombinations()
    {
        var expectedCombination = new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.LShift, null, KeyKind.Virtual),
            new Key(null, 'v', KeyKind.Text)
        });
        var parsed = KeyCombination.TryParse("LShift+v");
        var switched = KeyCombination.TryParse("v+LShift");

        Assert.AreEqual(expectedCombination, parsed);
        Assert.AreEqual(parsed, switched);
        Assert.AreEqual(expectedCombination.GetHashCode(), parsed.GetHashCode());
        Assert.AreEqual(parsed.GetHashCode(), switched.GetHashCode());

        expectedCombination = new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.LShift, null, KeyKind.Virtual),
            new Key(null, 'v', KeyKind.Text),
            new Key(VirtualKeyCode.Space, null, KeyKind.Virtual),
            new Key(null, 'b', KeyKind.Text),
        });
        parsed = KeyCombination.TryParse("LShift+v+Space+b");
        switched = KeyCombination.TryParse("Space+v+b+LShift");

        Assert.AreEqual(expectedCombination, parsed);
        Assert.AreEqual(parsed, switched);
        Assert.AreEqual(expectedCombination.GetHashCode(), parsed.GetHashCode());
        Assert.AreEqual(parsed.GetHashCode(), switched.GetHashCode());

        // Empty / null input
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse(""));
        // Only separator
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("+"));
        // Remaining separator
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("LShift+"));
        // Duplicate key
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("LShift+LShift"));
        // Too many keys
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("LShift+a+b+c+d"));
        // Empty key surrounded by valid keys
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("LShift+ +a"));
    }

    [TestMethod]
    public void KeyCombinationToString()
    {
        Assert.AreEqual("LShift+a", new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.LShift, null, KeyKind.Virtual),
            new Key(null, 'a', KeyKind.Text)
        }).ToString());
    }

}
