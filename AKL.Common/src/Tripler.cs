namespace AKL.Common;

using AKL.Core;

public class Tripler
{
    public static int Triple(int input)
    {
        return AklCoreNativeInterface.triple(input);
    }
}
