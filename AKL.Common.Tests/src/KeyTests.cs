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

        Assert.ThrowsException<ArgumentOutOfRangeException>(() => Key.TryParse("With Whitespace"));
        Assert.ThrowsException<ArgumentException>(() => Key.TryParse("Something"));
    }

}
