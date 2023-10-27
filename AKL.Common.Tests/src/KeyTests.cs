namespace AKL.Common.Tests;

[TestClass]
public class KeyTests
{

    [TestMethod]
    public void KeyCombinationToString()
    {
        Assert.AreEqual("Shift+a", new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.Shift, null, KeyKind.Virtual),
            new Key(null, 'a', KeyKind.Text)
        }).ToString());
    }

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

        Assert.ThrowsException<ArgumentOutOfRangeException>(() => Key.TryParse("With Whitespace"));
        Assert.ThrowsException<ArgumentException>(() => Key.TryParse("Something"));
    }

    [TestMethod]
    public void ParseKeyCombinations()
    {
        var expectedCombination = new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.Shift, null, KeyKind.Virtual),
            new Key(null, 'v', KeyKind.Text)
        });
        var parsed = KeyCombination.TryParse("Shift+v");
        var switched = KeyCombination.TryParse("v+Shift");

        Assert.AreEqual(expectedCombination, parsed);
        Assert.AreEqual(parsed, switched);
        Assert.AreEqual(expectedCombination.GetHashCode(), parsed.GetHashCode());
        Assert.AreEqual(parsed.GetHashCode(), switched.GetHashCode());

        expectedCombination = new KeyCombination(new Key[] {
            new Key(VirtualKeyCode.Shift, null, KeyKind.Virtual),
            new Key(null, 'v', KeyKind.Text),
            new Key(VirtualKeyCode.Space, null, KeyKind.Virtual),
            new Key(null, 'b', KeyKind.Text),
        });
        parsed = KeyCombination.TryParse("Shift+v+Space+b");
        switched = KeyCombination.TryParse("Space+v+b+Shift");

        Assert.AreEqual(expectedCombination, parsed);
        Assert.AreEqual(parsed, switched);
        Assert.AreEqual(expectedCombination.GetHashCode(), parsed.GetHashCode());
        Assert.AreEqual(parsed.GetHashCode(), switched.GetHashCode());

        // Empty / null input
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse(""));
        // Only separator
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("+"));
        // Remaining separator
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("Shift+"));
        // Duplicate key
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("Shift+Shift"));
        // Too many keys
        Assert.ThrowsException<ArgumentException>(() => KeyCombination.TryParse("Shift+a+b+c+d"));
    }

}
