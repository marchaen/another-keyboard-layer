using System.Collections.ObjectModel;
using ReactiveUI;

namespace AKL.Gui;

public class MainWindowViewModel : ReactiveObject
{

    public bool StartWithSystem { get; set; } = true;

    public string RawSwitchKey { get; set; } = "CapsLock";

    public string RawDefaultCombination { get; set; } = "Escape";

    public ObservableCollection<RawKeyMapping> RawKeyMappings { get; }

    public MainWindowViewModel()
    {
        RawKeyMappings = new() {
            new RawKeyMapping("h", "LeftArrow"),
            new RawKeyMapping("j", "DownArrow"),
            new RawKeyMapping("k", "UpArrow"),
            new RawKeyMapping("l", "RightArrow"),
            new RawKeyMapping("LControl + j", "PageUp"),
            new RawKeyMapping("LControl + k", "PageDown"),
            new RawKeyMapping("LShift + d", "Delete"),
            new RawKeyMapping("Back", "Delete"),
        };
    }

    public class RawKeyMapping
    {

        public string RawTargetCombination { get; set; }
        public string RawReplacementCombination { get; set; }

        public RawKeyMapping(string target, string replacement)
        {
            RawTargetCombination = target;
            RawReplacementCombination = replacement;
        }

    }

}
