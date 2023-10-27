namespace AKL.Common;

/// <summary>
///     Checks if another program with a specified identifier is already running
///     by trying to create an
///     <see href="https://learn.microsoft.com/en-us/dotnet/api/system.threading.eventwaithandle?view=net-7.0">
///         windows EventWaitHandle
///     </see> which is normally used to communicate with other threads on the
///     same machine with signals.
/// 
///     <seealso href="https://stackoverflow.com/questions/646480/is-using-a-mutex-to-prevent-multiple-instances-of-the-same-program-from-running"/>
/// </summary>
public class SingleInstanceChecker
{

    // Keep the event handles alive until the program exits.
    private static Dictionary<string, EventWaitHandle> eventHandles = new();

    /// <summary>
    ///     Checks if another program with the specified identifier is already
    ///     running by creating a windows wait handle.
    /// 
    ///     If the first program calls this method with id "foo" and then
    ///     another program gets started and also calls this method with "foo"
    ///     <c>false</c> will be returned.
    /// 
    ///     The event wait handles will not be preserved on shutdown.
    /// </summary>
    /// <param name="identifier">The identifier of the application.</param>
    /// <param name="currentUserOnly">
    ///     If a specific identifier should be globally unique or only for the
    ///     current user. If currentUserOnly is <c>true</c> each user will be
    ///     able to the application.
    /// </param>
    /// <returns>If another instance of a program is already running.</returns>
    public static bool IsOtherAlreadyRunning(string identifier, bool currentUserOnly = true)
    {
        if (eventHandles.ContainsKey(identifier))
            return false;

        identifier = "#" + identifier + "#instance";

        if (currentUserOnly)
            identifier = Environment.UserName + identifier;

        var eventHandle = new EventWaitHandle(
            false,
            EventResetMode.ManualReset,
            identifier,
            out bool createdSuccessfully
        );

        if (createdSuccessfully)
        {
            eventHandles[identifier] = eventHandle;
            return false;
        }

        return true;
    }

}
