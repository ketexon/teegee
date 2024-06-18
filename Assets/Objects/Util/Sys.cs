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
    private static extern IntPtr GetActiveWindow();

    [DllImport("Kernel32.dll")]
    private static extern IntPtr GetConsoleWindow();

    [DllImport("user32.dll")]
    static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("Kernel32.dll")]
    public static extern bool AllocConsole();

    [DllImport("Kernel32.dll")]
    public static extern bool FreeConsole();


    public const UInt32 StdOutputHandle = 0xFFFFFFF5;
    [DllImport("kernel32.dll")]
    public static extern IntPtr GetStdHandle(UInt32 nStdHandle);
    [DllImport("kernel32.dll")]
    public static extern void SetStdHandle(UInt32 nStdHandle, IntPtr handle);

    [DllImport("kernel32.dll")]
    public static extern bool CancelIo(IntPtr file);

    [DllImport("kernel32.dll",
            EntryPoint = "CreateFileW",
            SetLastError = true,
            CharSet = CharSet.Auto,
            CallingConvention = CallingConvention.StdCall)]
    private static extern IntPtr CreateFileW(
        string lpFileName,
        UInt32 dwDesiredAccess,
        UInt32 dwShareMode,
        IntPtr lpSecurityAttributes,
        UInt32 dwCreationDisposition,
        UInt32 dwFlagsAndAttributes,
        IntPtr hTemplateFile
    );

    private const UInt32 GENERIC_WRITE = 0x40000000;
    private const UInt32 GENERIC_READ = 0x80000000;
    private const UInt32 FILE_SHARE_READ = 0x00000001;
    private const UInt32 FILE_SHARE_WRITE = 0x00000002;
    private const UInt32 OPEN_EXISTING = 0x00000003;
    private const UInt32 FILE_ATTRIBUTE_NORMAL = 0x80;
    private const UInt32 ERROR_ACCESS_DENIED = 5;

    private const UInt32 ATTACH_PARRENT = 0xFFFFFFFF;

    const int SW_HIDE = 0;
    const int SW_NORMAL = 1;

    public static void SetWindowVisible(bool visible)
    {
        var hwnd = GetActiveWindow();
        ShowWindow(hwnd, visible ? SW_NORMAL : SW_HIDE);
    }

    private static (SafeFileHandle, FileStream) CreateFileStream(
        string name, 
        uint win32DesiredAccess, 
        uint win32ShareMode,
        FileAccess dotNetFileAccess
    ) 
    {
        var file = new SafeFileHandle(CreateFileW(name, win32DesiredAccess, win32ShareMode, IntPtr.Zero, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, IntPtr.Zero), true);
        if (!file.IsInvalid)
        {
            var fs = new FileStream(file, dotNetFileAccess);
            return (file, fs);
        }
        throw new NotImplementedException();
    }

    static SafeFileHandle stdoutHandle;
    static FileStream stdoutFS;
    static StreamWriter stdoutWriter;

    static SafeFileHandle stdinHandle;
    static FileStream stdinFS;
    static StreamReader stdinReader;

    public static void OpenConsole()
    {
        AllocConsole();
        (stdoutHandle, stdoutFS) = CreateFileStream("CONOUT$", GENERIC_WRITE, FILE_SHARE_WRITE, FileAccess.Write);
        if (stdoutFS != null)
        {
            stdoutWriter = new StreamWriter(stdoutFS) { AutoFlush = true };
            Console.SetOut(stdoutWriter);
            Console.SetError(stdoutWriter);
        }

        (stdinHandle, stdinFS) = CreateFileStream("CONIN$", GENERIC_READ, FILE_SHARE_READ, FileAccess.Read);
        if (stdoutFS != null)
        {
            stdinReader = new StreamReader(stdinFS);
            Console.SetIn(stdinReader);
        }
    }

    public static void CloseConsole()
    {
        stdoutWriter.Close();
        stdinReader.Close();

        stdoutFS.Close();
        stdinFS.Close();

        stdinHandle.Close();
        stdoutHandle.Close();

        FreeConsole();

        Console.SetIn(StreamReader.Null);
        Console.SetOut(StreamWriter.Null);
        Console.SetError(StreamWriter.Null);
    }

    public static void CancelStdIo()
    {
        CancelIo(stdoutHandle.DangerousGetHandle());
    }
}
