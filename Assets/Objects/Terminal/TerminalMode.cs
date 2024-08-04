#if UNITY_EDITOR_LINUX || UNITY_STANDALONE_LINUX
#define UNITY_LINUX
#endif

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
#   if UNITY_EDITOR_WIN
    const string TerminalExecutableRelativePath = "../terminal-client/target/debug/terminal-client.exe";
#   elif UNITY_EDITOR_LINUX
    const string TerminalExecutableRelativePath = "../terminal-client/target/debug/terminal-client";
#   endif

    string TerminalExecutablePath => Path.Combine(Application.dataPath, TerminalExecutableRelativePath);
#else
#   if UNITY_EDITOR_WIN
    const string TerminalExecutableRelativePath = "terminal-client.exe";
#   elif UNITY_EDITOR_LINUX
    const string TerminalExecutableRelativePath = "terminal-client";
#   endif

    string TerminalExecutablePath => Path.Combine(Application.streamingAssetsPath, TerminalExecutableRelativePath);
#endif

    Process process = null;
    IPC.Server server = new();

    public System.Action<IPC.IMessage> MessageEvent;
    public System.Action TerminateEvent;

    Queue<IPC.IMessage> messageQueue = new();

    CancellationTokenSource cancellationTokenSource = null;

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
        cancellationTokenSource = new();

        server.Start();
#if UNITY_LINUX
        Debug.Log(Sys.GetTerminalEmulator());
        process = new Process() {
            EnableRaisingEvents = true,
            StartInfo =
            {
                FileName = Sys.GetTerminalEmulator(),
                Arguments = $"-e '{TerminalExecutablePath}'"
            },
        };
#else
        process = new Process() {
            EnableRaisingEvents = true,
            StartInfo =
            {
                FileName = TerminalExecutablePath,
            },
        };
#endif
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
            Debug.Log("Reading cancelled via cancellation token.");
            // polling cancelled via cancellationTokenSource.Cancel()
        }
    }

    private void OnProcessExited(object sender, System.EventArgs e)
    {
        Debug.Log("Process exited.");

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
        Sys.SetWindowVisible(true);
    }

    void HideWindow()
    {
        Sys.SetWindowVisible(false);
    }

    void OnDestroy()
    {
        StopTerminal();
    }
}
