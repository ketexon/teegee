using Microsoft.Win32.SafeHandles;
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using System.Threading.Tasks;
using UnityEngine;

public static class Sys
{
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
        if(activeWindow == IntPtr.Zero)
        {
            activeWindow = GetActiveWindow();
        }
        ShowWindow(activeWindow, visible ? SW_RESTORE : SW_HIDE);
    }
}
