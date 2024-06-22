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
                await OnServerConnected();
            }
        });
    }

    async Task OnServerConnected()
    {
        var t = server.ReadExactlyAsync(4);
        if(!await t.WaitAsync(1000))
        {
            Debug.LogError("No data from client");
            return;
        }
        else if(t.Result.Length != 4)
        {
            Debug.LogError("Incomplete data from client");
            return;
        }
        var i = BitConverter.ToInt32(t.Result);
        Debug.Log(i);
    }


    void Update()
    {
        //if (process is not null && !connected)
        //{
        //    if (server.Accept())
        //    {
        //        Debug.Log("Connected");
        //        connected = true;
        //    }
        //}
        //if (pipeServer != null && pipeServer.CanRead && pipeServer.IsConnected)
        //{
        //    int b;
        //    while((b = pipeServer.ReadByte()) >= 0)
        //    {
        //        Debug.Log(b);
        //        pipeBuffer.Add((byte) b);
        //    }
        //}
    }

    private void OnProcessExited(object sender, System.EventArgs e)
    {
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
