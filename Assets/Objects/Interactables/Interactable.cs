using System.Collections;
using System.Collections.Generic;
using UnityEditor;
using UnityEngine;

public class Interactable : MonoBehaviour
{
    [SerializeField] List<Renderer> toonRenderers = new();

    bool highlighted = false;

    void Reset()
    {
        toonRenderers = new(GetComponentsInChildren<Renderer>());
    }

    public virtual void Interact() {}

    public void Highlight()
    {
        if (highlighted) return;
        highlighted = true;

        foreach (var r in toonRenderers)
        {
            r.material.SetFloat("_EnableOutline", 1.0f);
            r.material.SetShaderPassEnabled("Outline", true);
        }
    }
    public void Unhighlight() { 
        if(!highlighted) return;
        highlighted = false;

        foreach(var r in toonRenderers)
        {
            r.material.SetFloat("_EnableOutline", 0.0f);
            r.material.SetShaderPassEnabled("Outline", false);
        }
    }
}
