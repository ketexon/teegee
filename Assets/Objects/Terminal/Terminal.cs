using UnityEngine;

public class Terminal : Interactable
{
    [SerializeField] InitializeMessage args;

    public override void Interact()
    {
        base.Interact();

        TerminalMode.Instance.StartTerminal(args);
        TerminalMode.Instance.TerminateEvent += OnTerminate;
    }

    private void OnTerminateInternal(IMessage message)
    {
        TerminalMode.Instance.TerminateEvent -= OnTerminate;

        OnTerminate(message);
    }

    protected virtual void OnTerminate(IMessage message)
    {
        
    }
}