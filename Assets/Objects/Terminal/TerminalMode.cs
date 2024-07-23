using System;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.IO.Pipes;
using System.Net.Sockets;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
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
    IPC.Server server = new();

    public System.Action<IPC.IMessage> MessageEvent;
    public System.Action TerminateEvent;

    Queue<IPC.IMessage> messageQueue = new();

    CancellationTokenSource cancellationTokenSource = new();

    Task messageReaderTask = null;

    bool terminated = false;

    void Update()
    {
        lock (messageQueue)
        {
            while (messageQueue.TryDequeue(out var msg))
            {
                Debug.Log($"MESSAGE RECEIVED: {msg}");
                MessageEvent?.Invoke(msg);
            }
        }

        if (terminated)
        {
            TerminateEvent?.Invoke();
            terminated = false;
        }
    }

    public void StartTerminal(IPC.InitializeMessage initialize)
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
        messageReaderTask = Task.Run(async () =>
        {
            await OnServerStarted(initialize);
        });
    }

    async Task OnServerStarted(IPC.InitializeMessage initializeMessage)
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
            server.WriteMessage(initializeMessage);
        }
        try
        {
            while (true)
            {
                Debug.Log("Reading message...");
                var msg = await server.ReadMessageAsync(cancellationTokenSource.Token);
                Debug.Log("Message read.");
                lock (messageQueue)
                {
                    messageQueue.Enqueue(msg);
                }
                // if the process terminated, don't read any more messages
                if(process == null)
                {
                    break;
                }
            }
        }
        catch(TaskCanceledException)
        {
            // polling cancelled via cancellationTokenSource.Cancel()
        }
    }

    private void OnProcessExited(object sender, System.EventArgs e)
    {
        process = null;

        // if there is no more data, just immediately quit
        if(!server.DataAvailable)
        {
            cancellationTokenSource.Cancel();
        }
        // otherwise, wait (max 1s) for the messaging thread to finish
        // and, if it doesn't, force quit it.
        else
        {
            var task = Task.WhenAny(Task.Delay(1000), messageReaderTask);
            task.Wait();
            if(task.Result != messageReaderTask)
            {
                cancellationTokenSource.Cancel();
            }
        }

        ShowWindow();

        server.Stop();

        terminated = true;
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
