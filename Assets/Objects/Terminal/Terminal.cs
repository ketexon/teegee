using UnityEngine;
public class Terminal : Interactable
{
    [SerializeField] IPC.InitializeMessage args;


    public override void Interact()
    {
        base.Interact();

        TerminalMode.Instance.StartTerminal(args);
        TerminalMode.Instance.MessageEvent += OnMessage;
        TerminalMode.Instance.TerminateEvent += OnTerminate;
    }

    protected virtual void OnTerminate()
    {
        TerminalMode.Instance.MessageEvent -= OnMessage;
        TerminalMode.Instance.TerminateEvent -= OnTerminate;
    }

    protected virtual void OnMessage(IPC.IMessage message)
    {
        Debug.Log($"Message: {message}");
    }
}