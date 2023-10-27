using AKL.Common;

Console.WriteLine("Start virtual layer...");

// See verbatim text: https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/tokens/verbatim
var raw_config = @"
switch_key = ""CapsLock""
start_with_system = false
default_simulation_combination = """"
[mappings]
""a""=""o""
";

var configuration = AklConfiguration.FromString(raw_config);

var virtualLayer = new VirtualLayer(configuration);
virtualLayer.Update();

Console.ReadKey();
Console.WriteLine("Stopped.");
