namespace AKL.Common;

public class Key
{

    private VirtualKeyCode? virtualKey;
    private char? textKey;
    private KeyKind kind;

    public static Key TryParse(string raw)
    {
        // TODO: Think about parsing algorithm and implement it.
        return new Key();
    }

}

public enum KeyKind
{
    Text,
    Virtual
}

public static class VirtualKeyCodeParser
{

    public static VirtualKeyCode? Parse(string raw)
    {
        var successful = Enum.TryParse(raw, out VirtualKeyCode parsed);

        if (successful)
        {
            return parsed;
        }
        else
        {
            return null;
        }
    }

}

public enum VirtualKeyCode
{
    Back,
    Tab,
    Clear,
    Return,
    Shift,
    Control,
    Menu,
    Pause,
    Capital,
    Kana,
    Hangul,
    ImeOn,
    Junja,
    Final,
    Hanja,
    Kanji,
    ImeOff,
    Escape,
    Convert,
    Nonconvert,
    Accept,
    Modechange,
    Space,
    Prior,
    Next,
    End,
    Home,
    Left,
    Up,
    Right,
    Down,
    Select,
    Print,
    Execute,
    Snapshot,
    Insert,
    Delete,
    Help,
    LWin,
    RWin,
    Apps,
    Sleep,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    Multiply,
    Add,
    Separator,
    Subtract,
    Decimal,
    Divide,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Numlock,
    Scroll,
    LShift,
    RShift,
    LControl,
    RControl,
    LMenu,
    RMenu,
    BrowserBack,
    BrowserForward,
    BrowserRefresh,
    BrowserStop,
    BrowserSearch,
    BrowserFavorites,
    BrowserHome,
    VolumeMute,
    VolumeDown,
    VolumeUp,
    MediaNextTrack,
    MediaPrevTrack,
    MediaStop,
    MediaPlayPause,
    LaunchMail,
    LaunchMediaSelect,
    LaunchApp1,
    LaunchApp2,
    Processkey,
    Play,
    Zoom,
}
