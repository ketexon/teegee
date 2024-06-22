using UnityEngine;

public class Terminal : Interactable
{
    [SerializeField] TerminalArguments args;

    public override void Interact()
    {
        base.Interact();

        TerminalMode.Instance.StartTerminal(args);
    }
}