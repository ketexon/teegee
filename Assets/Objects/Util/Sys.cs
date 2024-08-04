using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

public static class Sys
{
#if UNITY_EDITOR_LINUX || UNITY_STANDALONE_LINUX
    public static string Which(string input)
    {
        var process = new System.Diagnostics.Process()
        {
            StartInfo = {
                UseShellExecute = false,
                FileName = "which",
                Arguments = input,
                RedirectStandardOutput = true,
            },
        };

        var outputBuilder = new StringBuilder();
        process.OutputDataReceived += (sender, args) =>
        {
            outputBuilder.Append(args.Data);
        };

        process.Start();
        process.BeginOutputReadLine();
        process.WaitForExit();
        process.CancelOutputRead();

        if (process.ExitCode != 0)
        {
            return null;
        }
        return outputBuilder.ToString();
    }

    static readonly string[] terminals = new string[] {
        "x-terminal-emulator",
        "mate-terminal",
        "gnome-terminal",
        "terminator",
        "xfce4-terminal",
        "urxvt",
        "rxvt",
        "termit",
        "Eterm",
        "aterm",
        "uxterm",
        "xterm",
        "roxterm",
        "termite",
        "lxterminal",
        "terminology",
        "st",
        "qterminal",
        "lilyterm",
        "tilix",
        "terminix",
        "konsole",
        "kitty",
        "guake",
        "tilda",
        "alacritty",
        "hyper",
        "wezterm",
        "rio",
    };

    static string selectedTerminal = null;

    public static string GetTerminalEmulator()
    {


        if(selectedTerminal != null) return selectedTerminal;
        foreach(var term in terminals){
            var which = Which(term);
            if (which != null){
                selectedTerminal = which;
                return term;
            }
        }
        return null;
    }

    [DllImport("libX11.so")]
    static extern IntPtr XOpenDisplay(string display_name);

    public static void SetWindowVisible(bool visible)
    {
#if UNITY_EDITOR
        Debug.Log($"Window Visible: {visible}");
#else

#endif
    }

#elif UNITY_EDITOR_WIN || UNITY_STANDALONE_WIN
    [DllImport("user32.dll")]
    static extern IntPtr GetActiveWindow();

    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    const int SW_HIDE = 0;
    const int SW_SHOW = 5;
    const int SW_MINIMIZE = 6;
    const int SW_RESTORE = 9;

    static IntPtr activeWindow = IntPtr.Zero;

    public static void SetWindowVisible(bool visible)
    {
#if UNITY_EDITOR
        Debug.Log($"Window Visible: {visible}");
#else
        if(activeWindow == IntPtr.Zero)
        {
            activeWindow = GetActiveWindow();
        }
        ShowWindow(activeWindow, visible ? SW_RESTORE : SW_HIDE);
#endif
    }
#endif
}
