using System.IO;
using UnityEngine;

using Process = System.Diagnostics.Process;

public class TerminalMode : MonoBehaviour
{
#if UNITY_EDITOR
    const string TerminalExecutableRelativePath = "../terminal-client/target/debug/terminal-client.exe";
    string TerminalExecutablePath => Path.Combine(Application.dataPath, TerminalExecutableRelativePath);
#else

#endif

    Process process = null;

    void Start()
    {
        StartTerminal();
    }

    void StartTerminal()
    {
        process = Process.Start(TerminalExecutablePath);
    }

    void StopTerminal()
    {
        
    }

    void OnThreadComplete()
    {
        
    }

    void OnDestroy()
    {
        
    }
}
