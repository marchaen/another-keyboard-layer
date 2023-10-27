namespace AKL.Common.Tests;

[TestClass]
public class AklConfigurationTests
{

    [TestMethod]
    public void TestThatDefaultConfigurationIsValid()
    {
        Assert.IsNotNull(AklConfiguration.FromString(File.ReadAllText("./default-config.toml")));
    }

    [TestMethod]
    public void TestPossibleParsingErrors()
    {
        var autoStart = "start_with_system = true\n";
        var switchKey = "switch_key = \"Control\"\n";
        var defaultCombination = "default_simulation_combination = \"LMenu\"\n";
        var mappings = "[mappings]\n";

        // Not valid toml
        Assert.ThrowsException<AklConfigurationParsingException>(() => AklConfiguration.FromString("Invalid toml input!"));
        // No mappings section
        Assert.ThrowsException<AklConfigurationParsingException>(() => AklConfiguration.FromString(autoStart + switchKey + defaultCombination));
        // No default combination 
        Assert.ThrowsException<AklConfigurationParsingException>(() => AklConfiguration.FromString(autoStart + switchKey + mappings));
        // No switch key
        Assert.ThrowsException<AklConfigurationParsingException>(() => AklConfiguration.FromString(autoStart + defaultCombination + mappings));
        // No auto start
        Assert.ThrowsException<AklConfigurationParsingException>(() => AklConfiguration.FromString(switchKey + defaultCombination + mappings));
    }

    [TestMethod]
    public void TestCorrectSerialization()
    {
        var original = AklConfiguration.FromString(File.ReadAllText("./default-config.toml"));
        var originalHash = original.GetHashCode();

        var fromSerialization = AklConfiguration.FromString(original.ToString());
        var fromSerializationHash = fromSerialization.GetHashCode();

        Assert.AreEqual(original, fromSerialization);
        Assert.AreEqual(originalHash, fromSerializationHash);
    }

    [TestMethod]
    public void TestEndToEndWithProvider()
    {
        var provider = AklConfigurationProvider.LoadFromFile(new FileInfo("./new-config-file.toml"));
        provider.SaveToFile();

        Assert.IsTrue(File.Exists("./new-config-file.toml"));
        File.Delete("./new-config-file.toml");
    }

}
