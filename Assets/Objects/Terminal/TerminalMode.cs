#if UNITY_EDITOR_LINUX || UNITY_STANDALONE_LINUX
#define UNITY_LINUX
#endif

using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
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
    public IPC.Server Server = new();

    public System.Action<IPC.IMessage> MessageEvent;
    public System.Action TerminateEvent;
    public System.Action ConnectedEvent;

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

        Server.Start();
#if UNITY_LINUX
        var terminalEmulator = Sys.GetTerminalEmulator();
        process = new Process() {
            EnableRaisingEvents = true,
            StartInfo =
            {
                FileName = terminalEmulator,
                Arguments = $"{Sys.GetExtraTerminalArguments(terminalEmulator)} -e '{TerminalExecutablePath}'"
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
        var task = Server.AcceptAsync();
        var completed = await Task.WhenAny(task, Task.Delay(1000));
        if (completed != task)
        {
            Debug.LogError("Could not connect");
        }
        else
        {
            Debug.Log("Connected");
            Server.WriteMessage(initializeMessage);
            ConnectedEvent?.Invoke();
        }
        try
        {
            while (true)
            {
                Debug.Log("Reading message...");
                var msg = await Server.ReadMessageAsync(cancellationTokenSource.Token);
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
        if(!Server.DataAvailable)
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

        Server.Stop();

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

    override protected void OnDestroy()
    {
        base.OnDestroy();
        StopTerminal();
    }
}
