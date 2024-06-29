using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.IO.Pipes;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using Unity.Burst.Intrinsics;
using UnityEngine;

using Process = System.Diagnostics.Process;

public class TerminalMode : SingletonMonoBehaviour<TerminalMode>
{
#if UNITY_EDITOR
    const string TerminalExecutableRelativePath = "../terminal-client/target/debug/terminal-client.exe";
    string TerminalExecutablePath => Path.Combine(Application.dataPath, TerminalExecutableRelativePath);
#else
    const string TerminalExecutableRelativePath = "terminal-client.exe";
    string TerminalExecutablePath => Path.Combine(Application.streamingAssetsPath, TerminalExecutableRelativePath);
#endif

    Process process = null;
    TerminalServer server = new();

    IMessage terminateMessage = null;
    public System.Action<IMessage> TerminateEvent;

    void Update()
    {
        // NOTE: This is done this way to 
        // make sure the terminate event is sent during
        // a unity lifecycle message.
        // Otherwise, it might be called between
        // messages, and this causes undefined behavior.
        if(terminateMessage != null)
        {
            TerminateEvent?.Invoke(terminateMessage);
            terminateMessage = null;
        }
    }

    public void StartTerminal(InitializeMessage initialize)
    {
        server.Start();
        process = new Process() { 
            EnableRaisingEvents = true,
            StartInfo =
            {
                FileName = TerminalExecutablePath,
            },
        };
        process.Exited += OnProcessExited;
        process.Start();
        HideWindow();
        Task.Run(async () =>
        {
            var task = server.AcceptAsync();
            var completed = await Task.WhenAny(task, Task.Delay(1000));
            if (completed != task)
            {
                Debug.LogError("Could not connect");
            }
            else
            {
                Debug.Log("Connected");
                server.WriteMessage(initialize);
            }
        });
    }

    private void OnProcessExited(object sender, System.EventArgs e)
    {
        Debug.Log("Process exited.");

        var task = server.ReadMessageAsync();
        if(!task.Wait(1000))
        {
            Debug.Log("Shutdown message not sent");
        }
        else
        {
            terminateMessage = task.Result;
        }

        Debug.Log($"Process exited: {process.ExitCode}");
        ShowWindow();
    
        process = null;

        server.Stop();
    }

    void StopTerminal()
    {
        if (process == null) return;
        process.CloseMainWindow();
        if (!process.WaitForExit(1000))
        {
            process.Kill();
        }
        process = null;
    }

    void ShowWindow()
    {
#if !UNITY_EDITOR
        Sys.SetWindowVisible(true);
#endif
        Debug.Log("SHOW WINDOW");
    }

    void HideWindow()
    {
#if !UNITY_EDITOR
        Sys.SetWindowVisible(false);
#endif
        Debug.Log("Hide WINDOW");
    }

    void OnDestroy()
    {
        StopTerminal();
    }
}
