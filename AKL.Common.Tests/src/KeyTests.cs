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

}
