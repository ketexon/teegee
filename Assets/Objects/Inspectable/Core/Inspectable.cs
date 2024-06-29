using System.Collections;
using System.Collections.Generic;
using UnityEngine;

public class Inspectable : Interactable
{
    [SerializeField] public GameObject InspectPrefab;

    public override void Interact()
    {
        base.Interact();

        InspectableOverlay.Instance.Inspect(this);
    }
}
