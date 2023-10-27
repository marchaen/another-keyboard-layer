namespace AKL.Common;

using AKL.Core;

public class Key
{

    private VirtualKeyCode? virtualKey;
    private char? textKey;
    private KeyKind kind;

    public Key(VirtualKeyCode? virtualKey, char? textKey, KeyKind kind)
    {
        this.virtualKey = virtualKey;
        this.textKey = textKey;
        this.kind = kind;
    }

    /// <summary>
    ///     Tries to parse raw as a keyboard key.
    /// 
    ///     Any keyboard key can be either a virtual or text key. If a key
    ///     produces a single character when pressed it's a text key otherwise
    ///     it's a virtual key. (The space key is the only exception!)
    /// </summary>
    /// <param name="raw">
    ///     The name of a virtual key or a single character to represent a text
    ///     key. So any single character input will be treated as a text key.
    /// </param>
    /// <exception cref="ArgumentOutOfRangeException">
    /// </exception>
    /// <exception cref="ArgumentException">
    ///     If no virtual key code with the specified name could be found or if 
    ///     the raw input contains any whitespace as defined
    ///     <a href="https://learn.microsoft.com/en-us/dotnet/api/system.char.iswhitespace?view=net-7.0#system-char-iswhitespace(system-char)">
    ///         here
    ///     </a>.
    /// </exception>
    /// <returns>A key that can further be used in key combinations.</returns>
    public static Key TryParse(string raw)
    {
        if (raw.Any(char.IsWhiteSpace))
        {
            throw new ArgumentException("A single key can't contain any whitespace.");
        }

        var virtualKey = VirtualKeyCodeParser.Parse(raw);

        if (virtualKey == null)
        {
            if (raw.Length != 1)
            {
                throw new ArgumentException(
                    $"Couldn't parse \"{raw}\" with a length of {raw.Length} as " + 
                    "a virtual nor plain text key."
                );
            }

            return new Key(null, raw[0], KeyKind.Text);
        }
        else
        {
            return new Key(virtualKey, null, KeyKind.Virtual);
        }
    }

    public override string ToString()
    {
        return kind switch
        {
            KeyKind.Virtual => this.virtualKey.ToString() ?? "This method can't fail.",
            _ => this.textKey.ToString() ?? "This method can't fail.",
        };
    }

    public override bool Equals(object? obj)
    {
        if (obj == null || GetType() != obj.GetType()) return false;

        Key other = (Key)obj;

        return this.virtualKey == other.virtualKey &&
            this.textKey == other.textKey &&
            this.kind == other.kind;
    }

    public override int GetHashCode()
    {
        unchecked {
            int hashcode = 1430287;

            if (this.virtualKey != null)
                hashcode = hashcode * 7302013 ^ (byte) this.virtualKey; 

            if (this.textKey != null)
                hashcode = hashcode * 7302013 ^ this.textKey.GetHashCode(); 

            return hashcode * 7302013 ^ (int) this.kind;
        }
    }
    
    internal FfiKey ToFfi() {
        var ffi = new FfiKey();

        switch (this.kind) {
            case KeyKind.Text:
                ffi.kind = FfiKeyKind.Text;
                ffi.text = Convert.ToUInt32(this.textKey);
                break;
            case KeyKind.Virtual:
                ffi.kind = FfiKeyKind.Virtual;
                ffi.named = (byte) (this.virtualKey ?? 0);
                break;
        }

        return ffi;
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

/// <summary>
///     Represents all keyboard keys which don't produce text when pressed.
///     <a href="https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes">
///         This resource
///     </a>
///     lists all existing virtual keys this enum is a smaller version of it 
///     with a couple keys renamed e. g. Capital => CapsLock, Menu => Alt, etc.
/// </summary> 
public enum VirtualKeyCode : byte {
    Back,
    Tab,
    Clear,
    Return,
    Shift,
    Control,
    Alt,
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
    ImeConvert,
    ImeNonconvert,
    ImeAccept,
    ImeModechange,
    Space,
    PageUp,
    PageDown,
    End,
    Home,
    LeftArrow,
    UpArrow,
    RightArrow,
    DownArrow,
    Select,
    Print,
    Execute,
    PrintScreen,
    Insert,
    Delete,
    Help,
    LMeta,
    RMeta,
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
    LAlt,
    RAlt,
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
