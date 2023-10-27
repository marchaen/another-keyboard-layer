namespace AKL.Common.Tests;

[TestClass]
public class SampleTest
{
    [TestMethod]
    public void Sample()
    {
        Assert.AreEqual("Hi", "Hi");
        Assert.AreNotEqual("Hello", "World!");
    }
}
